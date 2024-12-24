use super::{header_cache::StringCache, parser::RecordParser};
use std::{io::Read, thread};

pub enum ReadImpl<T: RecordParser> {
    SingleThreaded {
        reader: csv::Reader<Box<dyn Read>>,
        headers: Vec<&'static str>,
        null_string: Option<String>,
    },
    MultiThreaded {
        headers: Vec<&'static str>,
        receiver: kanal::Receiver<T::Output>,
        handle: Option<thread::JoinHandle<()>>,
    },
}

impl<T: RecordParser> ReadImpl<T> {
    #[inline]
    pub fn next(&mut self) -> Option<T::Output> {
        match self {
            Self::MultiThreaded {
                receiver, handle, ..
            } => match receiver.recv() {
                Ok(record) => Some(record),
                Err(_) => {
                    if let Some(handle) = handle.take() {
                        let _ = handle.join();
                    }
                    None
                }
            },
            Self::SingleThreaded {
                reader,
                headers,
                null_string,
            } => {
                let mut record = csv::StringRecord::new();
                match reader.read_record(&mut record) {
                    Ok(true) => Some(T::parse(headers, &record, null_string.as_deref())),
                    _ => None,
                }
            }
        }
    }

    #[inline]
    pub fn cleanup(&mut self) {
        match self {
            Self::MultiThreaded {
                receiver,
                handle,
                headers,
            } => {
                receiver.close();
                if let Some(handle) = handle.take() {
                    let _ = handle.join();
                }
                let _ = StringCache::clear(headers);
            }
            Self::SingleThreaded { headers, .. } => {
                let _ = StringCache::clear(headers);
            }
        }
    }
}
