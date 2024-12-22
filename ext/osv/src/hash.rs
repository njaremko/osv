use crate::utils::*;
use magnus::{block::Yield, rb_sys::AsRawValue, value::ReprValue, Error, Ruby, Value};
use std::{collections::VecDeque, fs::File, io::Read, os::fd::FromRawFd, thread};

/// Parses CSV data from a file and yields each row as a hash to the block.
pub fn parse_csv(
    ruby: &Ruby,
    rb_self: Value,
    args: &[Value],
) -> Result<Yield<impl Iterator<Item = std::collections::HashMap<String, String>>>, Error> {
    if !ruby.block_given() {
        return Ok(Yield::Enumerator(rb_self.enumeratorize("for_each", args)));
    }
    let (to_read, has_headers, delimiter) = parse_csv_args(args)?;

    let iter = BufferedRecordsAsHash::new(
        ruby,
        to_read,
        has_headers,
        delimiter.unwrap_or_else(|| ",".to_string()).as_bytes()[0],
        1000,
    );

    Ok(Yield::Iter(iter))
}

struct BufferedRecordsAsHash {
    reader: ReadImpl,
}

enum ReadMode {
    SingleThreaded,
    MultiThreaded,
}

enum ReadImpl {
    SingleThreaded {
        reader: csv::Reader<Box<dyn Read>>,
        headers: Vec<String>,
        buffer: VecDeque<std::collections::HashMap<String, String>>,
    },
    MultiThreaded {
        receiver: kanal::Receiver<std::collections::HashMap<String, String>>,
        handle: Option<thread::JoinHandle<()>>,
    },
}

impl BufferedRecordsAsHash {
    fn new(ruby: &Ruby, to_read: Value, has_headers: bool, delimiter: u8, buffer: usize) -> Self {
        let mut parallel = false;

        let readable = if to_read.is_kind_of(ruby.class_io()) {
            parallel = true;
            let fd = unsafe { rb_sys::rb_io_descriptor(to_read.as_raw()) };
            // let borrowed_fd = unsafe { BorrowedFd::borrow_raw(fd) };
            let file = unsafe { File::from_raw_fd(fd) };
            file
        } else {
            parallel = true;
            let path = to_read.to_r_string().unwrap().to_string().unwrap();
            let file = std::fs::File::open(&path)
                .map_err(|e| {
                    Error::new(
                        ruby.exception_runtime_error(),
                        format!("Failed to open file: {}", e),
                    )
                })
                .unwrap();
            file
        };

        let read_impl = if parallel {
            let mut reader = csv::ReaderBuilder::new()
                .has_headers(has_headers)
                .delimiter(delimiter)
                .from_reader(readable);

            let first_row = reader.headers().unwrap().clone();
            let num_fields = first_row.len();

            let headers: Vec<String> = if has_headers {
                first_row.iter().map(|h| h.to_string()).collect()
            } else {
                (0..num_fields).map(|i| format!("c{}", i)).collect()
            };

            let (sender, receiver) =
                kanal::bounded::<std::collections::HashMap<String, String>>(buffer);
            let handle = thread::spawn(move || {
                let mut record = csv::StringRecord::new();
                while let Ok(read) = reader.read_record(&mut record) {
                    if !read {
                        // Need to "forget" the file inside the reader. Since the file descriptor is managed by the Ruby runtime.
                        let file_to_forget = reader.into_inner();
                        std::mem::forget(file_to_forget);
                        break;
                    }
                    let row = record
                        .iter()
                        .enumerate()
                        .map(|(i, field)| (headers[i].clone(), field.to_string()))
                        .collect();

                    sender.send(row).unwrap();
                }
            });
            ReadImpl::MultiThreaded {
                receiver,
                handle: Some(handle),
            }
        } else {
            let readable: Box<dyn Read> = Box::new(readable);
            let mut reader = csv::ReaderBuilder::new()
                .has_headers(has_headers)
                .delimiter(delimiter)
                .from_reader(readable);

            let first_row = reader.headers().unwrap().clone();
            let num_fields = first_row.len();

            let headers: Vec<String> = if has_headers {
                first_row.iter().map(|h| h.to_string()).collect()
            } else {
                (0..num_fields).map(|i| format!("c{}", i)).collect()
            };
            ReadImpl::SingleThreaded {
                reader: reader,
                headers,
                buffer: VecDeque::with_capacity(1000),
            }
        };

        Self { reader: read_impl }
    }
}

impl Iterator for BufferedRecordsAsHash {
    type Item = std::collections::HashMap<String, String>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.reader {
            ReadImpl::MultiThreaded {
                receiver, handle, ..
            } => match receiver.recv() {
                Ok(record) => Some(record),
                Err(e) => {
                    if let Some(handle) = handle.take() {
                        handle.join().unwrap();
                    }
                    None
                }
            },
            ReadImpl::SingleThreaded {
                reader,
                headers,
                buffer,
                ..
            } => {
                let mut record = csv::StringRecord::new();
                if let Ok(read) = reader.read_record(&mut record) {
                    if !read {
                        return None;
                    }
                    Some(
                        record
                            .iter()
                            .enumerate()
                            .map(|(i, field)| (headers[i].clone(), field.to_string()))
                            .collect(),
                    )
                } else {
                    None
                }
            }
        }
    }
}
