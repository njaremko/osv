use super::header_cache::StringCacheKey;
use super::parser::RecordParser;
use super::{header_cache::StringCache, ruby_reader::SeekableRead};
use magnus::{Error, Ruby};
use std::io::{BufReader, Read};

/// Size of the internal buffer used for reading CSV records
pub(crate) const READ_BUFFER_SIZE: usize = 16384;

/// A reader that processes CSV records using a specified parser.
///
/// This struct implements Iterator to provide a streaming interface for CSV records.
pub struct RecordReader<T: RecordParser<'static>> {
    reader: csv::Reader<BufReader<Box<dyn SeekableRead>>>,
    headers: Vec<StringCacheKey>,
    null_string: Option<&'static str>,
    flexible_default: Option<String>,
    string_record: csv::StringRecord,
    parser: std::marker::PhantomData<T>,
}

impl<T: RecordParser<'static>> RecordReader<T> {
    /// Reads and processes headers from a CSV reader.
    ///
    /// # Arguments
    /// * `ruby` - Ruby VM context for error handling
    /// * `reader` - CSV reader instance
    /// * `has_headers` - Whether the CSV file contains headers
    ///
    /// # Returns
    /// A vector of header strings or generated column names if `has_headers` is false
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

        Ok(if has_headers {
            first_row.iter().map(String::from).collect()
        } else {
            (0..first_row.len()).map(|i| format!("c{i}")).collect()
        })
    }

    /// Creates a new RecordReader instance.
    pub(crate) fn new(
        reader: csv::Reader<BufReader<Box<dyn SeekableRead>>>,
        headers: Vec<StringCacheKey>,
        null_string: Option<&'static str>,
        flexible_default: Option<String>,
    ) -> Self {
        let headers_len = headers.len();
        Self {
            reader,
            headers,
            null_string,
            flexible_default,
            string_record: csv::StringRecord::with_capacity(READ_BUFFER_SIZE, headers_len),
            parser: std::marker::PhantomData,
        }
    }

    /// Attempts to read the next record, returning any errors encountered.
    fn try_next(&mut self) -> csv::Result<Option<T::Output>> {
        match self.reader.read_record(&mut self.string_record)? {
            true => Ok(Some(T::parse(
                &self.headers,
                &self.string_record,
                self.null_string,
                self.flexible_default
                    .as_ref()
                    .map(|s| std::borrow::Cow::Owned(s.clone())),
            ))),
            false => Ok(None),
        }
    }
}

impl<T: RecordParser<'static>> Iterator for RecordReader<T> {
    type Item = T::Output;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Note: We intentionally swallow errors here to maintain Iterator contract.
        // Errors can be handled by using try_next() directly if needed.
        self.try_next().ok().flatten()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None) // Cannot determine size without reading entire file
    }
}

impl<T: RecordParser<'static>> Drop for RecordReader<T> {
    #[inline]
    fn drop(&mut self) {
        // Intentionally ignore errors during cleanup as there's no meaningful way to handle them
        let _ = StringCache::clear(&self.headers);
    }
}
