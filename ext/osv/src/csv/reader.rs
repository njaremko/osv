use super::parser::RecordParser;
use magnus::{Error, Ruby};
use std::{io::Read, thread};

pub struct RecordReader<T: RecordParser> {
    pub(crate) reader: ReadImpl<T>,
}

#[allow(dead_code)]
pub enum ReadImpl<T: RecordParser> {
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

impl<T: RecordParser> RecordReader<T> {
    pub(crate) fn get_headers(
        ruby: &Ruby,
        reader: &mut csv::Reader<impl Read>,
        has_headers: bool,
    ) -> Result<Vec<String>, Error> {
        let first_row = reader
            .headers()
            .map_err(|e| {
                Error::new(
                    ruby.exception_runtime_error(),
                    format!("Failed to read headers: {e}"),
                )
            })?
            .clone();

        Ok(if has_headers {
            first_row.iter().map(String::from).collect()
        } else {
            (0..first_row.len()).map(|i| format!("c{i}")).collect()
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
            ReadImpl::SingleThreaded {
                reader,
                headers,
                null_string,
            } => {
                let mut record = csv::StringRecord::new();
                match reader.read_record(&mut record) {
                    Ok(true) => Some(T::parse(headers, &record, null_string)),
                    _ => None,
                }
            }
        }
    }
}
