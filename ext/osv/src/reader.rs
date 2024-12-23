use crate::utils::*;
use magnus::{
    block::Yield, rb_sys::AsRawValue, value::ReprValue, Error, RClass, RString, Ruby, Value,
};
use std::{collections::HashMap, fs::File, io::Read, os::fd::FromRawFd, thread};

/// Parses CSV data from a file and yields each row as a hash to the block.
pub fn parse_csv(
    ruby: &Ruby,
    rb_self: Value,
    args: &[Value],
) -> Result<Yield<impl Iterator<Item = HashMap<String, String>>>, Error> {
    if !ruby.block_given() {
        return Ok(Yield::Enumerator(rb_self.enumeratorize("for_each", args)));
    }
    let (to_read, has_headers, delimiter) = parse_csv_args(args)?;

    let iter = RecordReader::<HashMap<String, String>>::new(
        ruby,
        to_read,
        has_headers,
        delimiter.unwrap_or_else(|| ",".to_string()).as_bytes()[0],
        1000,
    )?;

    Ok(Yield::Iter(iter))
}

pub fn parse_compat(
    ruby: &Ruby,
    rb_self: Value,
    args: &[Value],
) -> Result<Yield<impl Iterator<Item = Vec<String>>>, Error> {
    if !ruby.block_given() {
        return Ok(Yield::Enumerator(
            rb_self.enumeratorize("for_each_compat", args),
        ));
    }
    let (to_read, has_headers, delimiter) = parse_csv_args(args)?;

    let iter = RecordReader::<Vec<String>>::new(
        ruby,
        to_read,
        has_headers,
        delimiter.unwrap_or_else(|| ",".to_string()).as_bytes()[0],
        1000,
    )?;

    Ok(Yield::Iter(iter))
}

pub trait RecordParser {
    type Output;

    fn parse(headers: &[String], record: &csv::StringRecord) -> Self::Output;
}

impl RecordParser for HashMap<String, String> {
    type Output = Self;

    fn parse(headers: &[String], record: &csv::StringRecord) -> Self::Output {
        let capacity = headers.len();
        let mut map = HashMap::with_capacity(capacity);
        for (i, field) in record.iter().enumerate() {
            map.insert(headers[i].to_owned(), field.to_string());
        }
        map
    }
}

impl RecordParser for Vec<String> {
    type Output = Self;

    fn parse(_headers: &[String], record: &csv::StringRecord) -> Self::Output {
        let mut output = Vec::with_capacity(record.len());
        for field in record.iter() {
            output.push(field.to_string());
        }
        output
    }
}

struct RecordReader<T: RecordParser> {
    reader: ReadImpl<T>,
}

#[allow(dead_code)]
enum ReadImpl<T: RecordParser> {
    SingleThreaded {
        reader: csv::Reader<Box<dyn Read + Send + 'static>>,
        headers: Vec<String>,
    },
    MultiThreaded {
        receiver: kanal::Receiver<T::Output>,
        handle: Option<thread::JoinHandle<()>>,
    },
}

impl<T: RecordParser + Send + 'static> RecordReader<T> {
    fn new(
        ruby: &Ruby,
        to_read: Value,
        has_headers: bool,
        delimiter: u8,
        buffer: usize,
    ) -> Result<Self, Error> {
        let string_io: RClass = ruby.eval("StringIO").map_err(|e| {
            Error::new(
                ruby.exception_runtime_error(),
                format!("Failed to get StringIO class: {}", e),
            )
        })?;

        let readable: Box<dyn Read + Send + 'static> = if to_read.is_kind_of(string_io) {
            let string: RString = to_read.funcall("string", ()).map_err(|e| {
                Error::new(
                    ruby.exception_runtime_error(),
                    format!("Failed to get string from StringIO: {}", e),
                )
            })?;
            let content = string.to_string().map_err(|e| {
                Error::new(
                    ruby.exception_runtime_error(),
                    format!("Failed to convert string to Rust String: {}", e),
                )
            })?;
            Box::new(std::io::Cursor::new(content))
        } else if to_read.is_kind_of(ruby.class_io()) {
            let fd = unsafe { rb_sys::rb_io_descriptor(to_read.as_raw()) };
            let file = unsafe { File::from_raw_fd(fd) };
            Box::new(file)
        } else {
            let path = to_read
                .to_r_string()
                .map_err(|e| {
                    Error::new(
                        ruby.exception_runtime_error(),
                        format!("Failed to convert path to string: {}", e),
                    )
                })?
                .to_string()
                .map_err(|e| {
                    Error::new(
                        ruby.exception_runtime_error(),
                        format!("Failed to convert RString to Rust String: {}", e),
                    )
                })?;
            let file = std::fs::File::open(&path).map_err(|e| {
                Error::new(
                    ruby.exception_runtime_error(),
                    format!("Failed to open file: {}", e),
                )
            })?;
            Box::new(file)
        };

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(has_headers)
            .delimiter(delimiter)
            .from_reader(readable);

        let headers = Self::get_headers(&mut reader, has_headers)?;
        let headers_clone = headers.clone();

        let (sender, receiver) = kanal::bounded(buffer);
        let handle = thread::spawn(move || {
            let mut record = csv::StringRecord::new();
            while let Ok(read) = reader.read_record(&mut record) {
                if !read {
                    let file_to_forget = reader.into_inner();
                    std::mem::forget(file_to_forget);
                    break;
                }
                let row = T::parse(&headers_clone, &record);
                if sender.send(row).is_err() {
                    break;
                }
            }
        });

        let read_impl = ReadImpl::MultiThreaded {
            receiver,
            handle: Some(handle),
        };

        Ok(Self { reader: read_impl })
    }

    fn get_headers(
        reader: &mut csv::Reader<impl Read>,
        has_headers: bool,
    ) -> Result<Vec<String>, Error> {
        let first_row = reader
            .headers()
            .map_err(|e| {
                Error::new(
                    magnus::exception::runtime_error(),
                    format!("Failed to read headers: {}", e),
                )
            })?
            .clone();
        let num_fields = first_row.len();

        Ok(if has_headers {
            first_row.iter().map(|h| h.to_string()).collect()
        } else {
            (0..num_fields).map(|i| format!("c{}", i)).collect()
        })
    }
}

impl<T: RecordParser> Iterator for RecordReader<T> {
    type Item = T::Output;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.reader {
            ReadImpl::MultiThreaded { receiver, handle } => match receiver.recv() {
                Ok(record) => Some(record),
                Err(_) => {
                    if let Some(handle) = handle.take() {
                        let _ = handle.join();
                    }
                    None
                }
            },
            ReadImpl::SingleThreaded { reader, headers } => {
                let mut record = csv::StringRecord::new();
                match reader.read_record(&mut record) {
                    Ok(true) => Some(T::parse(headers, &record)),
                    _ => None,
                }
            }
        }
    }
}
