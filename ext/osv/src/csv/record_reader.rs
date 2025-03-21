use super::builder::ReaderError;
use super::header_cache::StringCacheKey;
use super::parser::{CsvRecordType, RecordParser};
use super::ruby_reader::RubyReader;
use magnus::{Error, Ruby};
use std::borrow::Cow;
use std::io::{BufReader, Read};

/// Size of the internal buffer used for reading CSV records
pub(crate) const READ_BUFFER_SIZE: usize = 16384;

/// A reader that processes CSV records using a specified parser.
///
/// This struct implements Iterator to provide a streaming interface for CSV records.
pub struct RecordReader<'a, 'r, T: RecordParser<'a>> {
    handle: &'r Ruby,
    reader: csv::Reader<BufReader<RubyReader>>,
    headers: Vec<StringCacheKey>,
    null_string: Option<Cow<'a, str>>,
    string_record: CsvRecordType,
    parser: std::marker::PhantomData<T>,
    ignore_null_bytes: bool,
}

impl<'a, 'r, T: RecordParser<'a>> RecordReader<'a, 'r, T> {
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
        lossy: bool,
    ) -> Result<Vec<String>, Error> {
        let headers = if lossy {
            let first_row = reader.byte_headers().map_err(|e| {
                Error::new(
                    ruby.exception_runtime_error(),
                    format!("Failed to read headers: {e}"),
                )
            })?;
            if has_headers {
                first_row
                    .iter()
                    .map(String::from_utf8_lossy)
                    .map(|x| x.to_string())
                    .collect()
            } else {
                (0..first_row.len()).map(|i| format!("c{i}")).collect()
            }
        } else {
            let first_row = reader.headers().map_err(|e| {
                Error::new(
                    ruby.exception_runtime_error(),
                    format!("Failed to read headers: {e}"),
                )
            })?;
            if has_headers {
                first_row.iter().map(String::from).collect()
            } else {
                (0..first_row.len()).map(|i| format!("c{i}")).collect()
            }
        };

        Ok(headers)
    }

    /// Creates a new RecordReader instance.
    pub(crate) fn new(
        handle: &'r Ruby,
        reader: csv::Reader<BufReader<RubyReader>>,
        headers: Vec<StringCacheKey>,
        null_string: Option<Cow<'a, str>>,
        ignore_null_bytes: bool,
        lossy: bool,
    ) -> Self {
        let headers_len = headers.len();
        Self {
            handle,
            reader,
            headers,
            null_string,
            string_record: if lossy {
                CsvRecordType::Byte(csv::ByteRecord::with_capacity(
                    READ_BUFFER_SIZE,
                    headers_len,
                ))
            } else {
                CsvRecordType::String(csv::StringRecord::with_capacity(
                    READ_BUFFER_SIZE,
                    headers_len,
                ))
            },
            parser: std::marker::PhantomData,
            ignore_null_bytes,
        }
    }

    /// Attempts to read the next record, returning any errors encountered.
    fn try_next(&mut self) -> Result<Option<T::Output>, ReaderError> {
        let record = match self.string_record {
            CsvRecordType::String(ref mut record) => self.reader.read_record(record),
            CsvRecordType::Byte(ref mut record) => self.reader.read_byte_record(record),
        }?;
        if record {
            Ok(Some(T::parse(
                self.handle,
                &self.headers,
                &self.string_record,
                self.null_string.clone(),
                self.ignore_null_bytes,
            )?))
        } else {
            Ok(None)
        }
    }
}

impl<'a, T: RecordParser<'a>> Iterator for RecordReader<'a, '_, T> {
    type Item = Result<T::Output, ReaderError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.try_next() {
            Ok(Some(record)) => Some(Ok(record)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None) // Cannot determine size without reading entire file
    }
}
