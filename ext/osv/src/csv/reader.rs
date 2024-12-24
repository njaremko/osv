use super::{parser::RecordParser, read_impl::ReadImpl};
use magnus::{Error, Ruby};
use std::{borrow::Cow, io::Read};

pub struct RecordReader<T: RecordParser> {
    pub(crate) reader: ReadImpl<T>,
}

impl<T: RecordParser> RecordReader<T> {
    #[inline]
    pub(crate) fn get_headers(
        ruby: &Ruby,
        reader: &mut csv::Reader<impl Read>,
        has_headers: bool,
    ) -> Result<Vec<String>, Error> {
        let first_row = reader.headers().map_err(|e| {
            Error::new(
                ruby.exception_runtime_error(),
                Cow::Owned(format!("Failed to read headers: {e}")),
            )
        })?;

        Ok(if has_headers {
            let mut headers = Vec::with_capacity(first_row.len());
            headers.extend(first_row.iter().map(String::from));
            headers
        } else {
            let mut headers = Vec::with_capacity(first_row.len());
            headers.extend((0..first_row.len()).map(|i| format!("c{i}")));
            headers
        })
    }
}

impl<T: RecordParser> Iterator for RecordReader<T> {
    type Item = T::Output;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.reader.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // We can't know the exact size without reading the whole file
        (0, None)
    }
}

impl<T: RecordParser> Drop for RecordReader<T> {
    #[inline]
    fn drop(&mut self) {
        self.reader.cleanup();
    }
}
