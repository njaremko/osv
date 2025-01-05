use super::header_cache::StringCacheKey;
use super::parser::RecordParser;
use super::{header_cache::StringCache, ruby_reader::SeekableRead};
use magnus::{Error, Ruby};
use std::borrow::Cow;
use std::io::BufReader;
use std::io::Read;
use std::marker::PhantomData;

pub(crate) const READ_BUFFER_SIZE: usize = 16384;

pub struct RecordReader<'a, T: RecordParser<'a>> {
    reader: csv::Reader<BufReader<Box<dyn SeekableRead>>>,
    headers: Vec<StringCacheKey>,
    null_string: Option<String>,
    flexible_default: Option<Cow<'a, str>>,
    string_record: csv::StringRecord,
    _phantom: PhantomData<T>,
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

    pub(crate) fn new(
        reader: csv::Reader<BufReader<Box<dyn SeekableRead>>>,
        headers: Vec<StringCacheKey>,
        null_string: Option<String>,
        flexible_default: Option<&'a str>,
    ) -> Self {
        let headers_len = headers.len();
        Self {
            reader,
            headers,
            null_string,
            flexible_default: flexible_default.map(Cow::Borrowed),
            string_record: csv::StringRecord::with_capacity(READ_BUFFER_SIZE, headers_len),
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: RecordParser<'a>> Iterator for RecordReader<'a, T> {
    type Item = T::Output;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.read_record(&mut self.string_record) {
            Ok(true) => Some(T::parse(
                &self.headers,
                &self.string_record,
                self.null_string.as_deref(),
                self.flexible_default.clone(),
            )),
            Ok(false) => None,
            Err(_e) => None,
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
        let _ = StringCache::clear(&self.headers);
    }
}
