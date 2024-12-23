use crate::utils::*;
use magnus::{
    block::Yield, rb_sys::AsRawValue, value::ReprValue, Error, IntoValue, RClass, RString, Ruby,
    Value,
};
use std::{
    collections::HashMap, fs::File, io::Read, marker::PhantomData, os::fd::FromRawFd, thread,
};

#[derive(Debug)]
pub enum CsvRecord {
    Vec(Vec<Option<String>>),
    Map(HashMap<String, Option<String>>),
}

impl IntoValue for CsvRecord {
    fn into_value_with(self, handle: &Ruby) -> Value {
        match self {
            CsvRecord::Vec(vec) => vec.into_value_with(handle),
            CsvRecord::Map(map) => map.into_value_with(handle),
        }
    }
}

/// Parses CSV data from a file and yields each row as a hash to the block.
pub fn parse_csv(
    ruby: &Ruby,
    rb_self: Value,
    args: &[Value],
) -> Result<Yield<impl Iterator<Item = CsvRecord>>, Error> {
    if !ruby.block_given() {
        return Ok(Yield::Enumerator(rb_self.enumeratorize("for_each", args)));
    }

    let CsvArgs {
        to_read,
        has_headers,
        delimiter,
        quote_char,
        null_string,
        buffer_size,
        result_type,
    } = parse_csv_args(args)?;

    let iter: Box<dyn Iterator<Item = CsvRecord>> = match result_type.as_str() {
        "hash" => Box::new(
            RecordReaderBuilder::<HashMap<String, Option<String>>>::new(ruby, to_read)
                .has_headers(has_headers)
                .delimiter(delimiter)
                .quote_char(quote_char)
                .null_string(null_string)
                .buffer(buffer_size)
                .build()?
                .map(CsvRecord::Map),
        ),
        "array" => Box::new(
            RecordReaderBuilder::<Vec<Option<String>>>::new(ruby, to_read)
                .has_headers(has_headers)
                .delimiter(delimiter)
                .quote_char(quote_char)
                .null_string(null_string)
                .buffer(buffer_size)
                .build()?
                .map(CsvRecord::Vec),
        ),
        _ => {
            return Err(Error::new(
                ruby.exception_runtime_error(),
                "Invalid result type",
            ))
        }
    };

    Ok(Yield::Iter(iter))
}

pub trait RecordParser {
    type Output;

    fn parse(headers: &[String], record: &csv::StringRecord, null_string: &str) -> Self::Output;
}

impl RecordParser for HashMap<String, Option<String>> {
    type Output = Self;

    fn parse(headers: &[String], record: &csv::StringRecord, null_string: &str) -> Self::Output {
        let capacity = headers.len();
        let mut map = HashMap::with_capacity(capacity);
        for (i, field) in record.iter().enumerate() {
            if field == null_string {
                map.insert(headers[i].to_owned(), None);
            } else {
                map.insert(headers[i].to_owned(), Some(field.to_string()));
            }
        }
        map
    }
}

impl RecordParser for Vec<Option<String>> {
    type Output = Self;

    fn parse(_headers: &[String], record: &csv::StringRecord, null_string: &str) -> Self::Output {
        let mut output = Vec::with_capacity(record.len());
        for field in record.iter() {
            if field == null_string {
                output.push(None);
            } else {
                output.push(Some(field.to_string()));
            }
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
        null_string: String,
    },
    MultiThreaded {
        receiver: kanal::Receiver<T::Output>,
        handle: Option<thread::JoinHandle<()>>,
    },
}

impl<T: RecordParser + Send + 'static> RecordReader<T> {
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

struct RecordReaderBuilder<'a, T: RecordParser + Send + 'static> {
    ruby: &'a Ruby,
    to_read: Value,
    has_headers: bool,
    delimiter: u8,
    quote_char: u8,
    null_string: String,
    buffer: usize,
    _phantom: PhantomData<T>,
}

impl<'a, T: RecordParser + Send + 'static> RecordReaderBuilder<'a, T> {
    fn new(ruby: &'a Ruby, to_read: Value) -> Self {
        Self {
            ruby,
            to_read,
            has_headers: true,
            delimiter: b',',
            quote_char: b'"',
            null_string: String::new(),
            buffer: 1000,
            _phantom: PhantomData,
        }
    }

    fn has_headers(mut self, has_headers: bool) -> Self {
        self.has_headers = has_headers;
        self
    }

    fn delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    fn quote_char(mut self, quote_char: u8) -> Self {
        self.quote_char = quote_char;
        self
    }

    fn null_string(mut self, null_string: String) -> Self {
        self.null_string = null_string;
        self
    }

    fn buffer(mut self, buffer: usize) -> Self {
        self.buffer = buffer;
        self
    }

    fn build(self) -> Result<RecordReader<T>, Error> {
        let string_io: RClass = self.ruby.eval("StringIO").map_err(|e| {
            Error::new(
                self.ruby.exception_runtime_error(),
                format!("Failed to get StringIO class: {}", e),
            )
        })?;

        let readable: Box<dyn Read + Send + 'static> = if self.to_read.is_kind_of(string_io) {
            let string: RString = self.to_read.funcall("string", ()).map_err(|e| {
                Error::new(
                    self.ruby.exception_runtime_error(),
                    format!("Failed to get string from StringIO: {}", e),
                )
            })?;
            let content = string.to_string().map_err(|e| {
                Error::new(
                    self.ruby.exception_runtime_error(),
                    format!("Failed to convert string to Rust String: {}", e),
                )
            })?;
            Box::new(std::io::Cursor::new(content))
        } else if self.to_read.is_kind_of(self.ruby.class_io()) {
            let fd = unsafe { rb_sys::rb_io_descriptor(self.to_read.as_raw()) };
            let file = unsafe { File::from_raw_fd(fd) };
            Box::new(file)
        } else {
            let path = self
                .to_read
                .to_r_string()
                .map_err(|e| {
                    Error::new(
                        self.ruby.exception_runtime_error(),
                        format!("Failed to convert path to string: {}", e),
                    )
                })?
                .to_string()
                .map_err(|e| {
                    Error::new(
                        self.ruby.exception_runtime_error(),
                        format!("Failed to convert RString to Rust String: {}", e),
                    )
                })?;
            let file = std::fs::File::open(&path).map_err(|e| {
                Error::new(
                    self.ruby.exception_runtime_error(),
                    format!("Failed to open file: {}", e),
                )
            })?;
            Box::new(file)
        };

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(self.has_headers)
            .delimiter(self.delimiter)
            .quote(self.quote_char)
            .from_reader(readable);

        let headers = RecordReader::<T>::get_headers(&mut reader, self.has_headers)?;
        let headers_clone = headers.clone();
        let null_string = self.null_string;

        let (sender, receiver) = kanal::bounded(self.buffer);
        let handle = thread::spawn(move || {
            let mut record = csv::StringRecord::new();
            while let Ok(read) = reader.read_record(&mut record) {
                if !read {
                    let file_to_forget = reader.into_inner();
                    std::mem::forget(file_to_forget);
                    break;
                }
                let row = T::parse(&headers_clone, &record, &null_string);
                if sender.send(row).is_err() {
                    break;
                }
            }
        });

        let read_impl = ReadImpl::MultiThreaded {
            receiver,
            handle: Some(handle),
        };

        Ok(RecordReader::<T> { reader: read_impl })
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
            ReadImpl::SingleThreaded {
                reader,
                headers,
                null_string,
            } => {
                let mut record = csv::StringRecord::new();
                match reader.read_record(&mut record) {
                    Ok(true) => Some(T::parse(headers, &record, &null_string)),
                    _ => None,
                }
            }
        }
    }
}
