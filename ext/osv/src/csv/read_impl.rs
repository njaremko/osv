use super::{header_cache::StringCache, parser::RecordParser};
use std::{io::Read, thread};

pub(crate) const READ_BUFFER_SIZE: usize = 16384;

pub enum ReadImpl<'a, T: RecordParser> {
    SingleThreaded {
        reader: csv::Reader<Box<dyn Read + 'a>>,
        headers: Vec<&'static str>,
        null_string: Option<String>,
        flexible_default: Option<String>,
        string_record: csv::StringRecord,
    },
    MultiThreaded {
        headers: Vec<&'static str>,
        receiver: kanal::Receiver<T::Output>,
        handle: Option<thread::JoinHandle<()>>,
    },
}

impl<'a, T: RecordParser> Iterator for ReadImpl<'a, T> {
    type Item = T::Output;

    #[inline]
    fn next(&mut self) -> Option<T::Output> {
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
                flexible_default,
                ref mut string_record,
            } => match reader.read_record(string_record) {
                Ok(true) => Some(T::parse(
                    headers,
                    &string_record,
                    null_string.as_deref(),
                    flexible_default.as_deref(),
                )),
                Ok(false) => None,
                Err(_e) => None,
            },
        }
    }
}

impl<'a, T: RecordParser> ReadImpl<'a, T> {
    #[inline]
    pub fn cleanup(&mut self) {
        match self {
            Self::MultiThreaded {
                receiver,
                handle,
                headers,
                ..
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
