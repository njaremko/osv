use super::parser::RecordParser;
use super::{header_cache::StringCache, ruby_reader::SeekableRead};
use magnus::{Error, Ruby};
use std::io::BufReader;
use std::{borrow::Cow, io::Read, thread};

pub(crate) const READ_BUFFER_SIZE: usize = 16384;

pub struct RecordReader<'a, T: RecordParser<'a>> {
    inner: ReaderImpl<'a, T>,
}

enum ReaderImpl<'a, T: RecordParser<'a>> {
    SingleThreaded {
        reader: csv::Reader<BufReader<Box<dyn SeekableRead>>>,
        headers: Vec<&'static str>,
        null_string: Option<String>,
        flexible_default: Option<Cow<'a, str>>,
        string_record: csv::StringRecord,
    },
    MultiThreaded {
        headers: Vec<&'static str>,
        receiver: kanal::Receiver<T::Output>,
        handle: Option<thread::JoinHandle<()>>,
    },
}

impl<'a, T: RecordParser<'a>> RecordReader<'a, T> {
    #[inline]
    pub(crate) fn get_headers(
        ruby: &Ruby,
        reader: &mut csv::Reader<impl Read>,
        has_headers: bool,
    ) -> Result<Vec<String>, Error> {
        let first_row = reader.headers().map_err(|e| {
            Error::new(
                ruby.exception_runtime_error(),
                format!("Failed to read headers: {e}"),
            )
        })?;

        let mut headers = Vec::with_capacity(first_row.len());
        if has_headers {
            headers.extend(first_row.iter().map(String::from));
        } else {
            headers.extend((0..first_row.len()).map(|i| format!("c{i}")));
        }
        Ok(headers)
    }

    pub(crate) fn new_single_threaded(
        reader: csv::Reader<BufReader<Box<dyn SeekableRead>>>,
        headers: Vec<&'static str>,
        null_string: Option<String>,
        flexible_default: Option<&'a str>,
    ) -> Self {
        let headers_len = headers.len();
        Self {
            inner: ReaderImpl::SingleThreaded {
                reader,
                headers,
                null_string,
                flexible_default: flexible_default.map(|s| Cow::Borrowed(s)),
                string_record: csv::StringRecord::with_capacity(READ_BUFFER_SIZE, headers_len),
            },
        }
    }
}

impl<T: RecordParser<'static> + Send> RecordReader<'static, T> {
    pub(crate) fn new_multi_threaded(
        mut reader: csv::Reader<Box<dyn Read + Send + 'static>>,
        headers: Vec<&'static str>,
        buffer_size: usize,
        null_string: Option<String>,
        flexible_default: Option<&'static str>,
    ) -> Self {
        let (sender, receiver) = kanal::bounded(buffer_size);
        let headers_for_thread = headers.clone();

        let handle = thread::spawn(move || {
            let mut record =
                csv::StringRecord::with_capacity(READ_BUFFER_SIZE, headers_for_thread.len());
            while let Ok(true) = reader.read_record(&mut record) {
                let row = T::parse(
                    &headers_for_thread,
                    &record,
                    null_string.as_deref(),
                    flexible_default.map(|s| Cow::Borrowed(s)),
                );
                if sender.send(row).is_err() {
                    break;
                }
            }
        });

        Self {
            inner: ReaderImpl::MultiThreaded {
                headers,
                receiver,
                handle: Some(handle),
            },
        }
    }
}

impl<'a, T: RecordParser<'a>> Iterator for RecordReader<'a, T> {
    type Item = T::Output;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            ReaderImpl::MultiThreaded {
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
            ReaderImpl::SingleThreaded {
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
                    flexible_default.clone(),
                )),
                Ok(false) => None,
                Err(_e) => None,
            },
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // We can't know the exact size without reading the whole file
        (0, None)
    }
}

impl<'a, T: RecordParser<'a>> Drop for RecordReader<'a, T> {
    #[inline]
    fn drop(&mut self) {
        match &mut self.inner {
            ReaderImpl::MultiThreaded {
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
            ReaderImpl::SingleThreaded { headers, .. } => {
                let _ = StringCache::clear(headers);
            }
        }
    }
}
