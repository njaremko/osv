use super::{parser::RecordParser, read_impl::ReadImpl};
use magnus::{Error, Ruby};
use std::io::Read;

pub struct RecordReader<'a, T: RecordParser> {
    pub(crate) reader: ReadImpl<'a, T>,
}

impl<'a, T: RecordParser> RecordReader<'a, T> {
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
}

impl<'a, T: RecordParser> Iterator for RecordReader<'a, T> {
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

impl<'a, T: RecordParser> Drop for RecordReader<'a, T> {
    #[inline]
    fn drop(&mut self) {
        self.reader.cleanup();
    }
}
