use super::{
    header_cache::{CacheError, StringCache},
    parser::RecordParser,
    record_reader::{RecordReader, READ_BUFFER_SIZE},
    ruby_reader::{build_ruby_reader, SeekableRead},
    ForgottenFileHandle,
};
use flate2::read::GzDecoder;
use magnus::{rb_sys::AsRawValue, value::ReprValue, Error as MagnusError, RString, Ruby, Value};
use std::{
    borrow::Cow,
    fmt::Debug,
    fs::File,
    io::{self, BufReader, Read},
    marker::PhantomData,
    mem::ManuallyDrop,
    os::fd::FromRawFd,
};

use thiserror::Error;

/// Errors that can occur when building a RecordReader
#[derive(Error, Debug)]
pub enum ReaderError {
    #[error("Failed to get file descriptor: {0}")]
    FileDescriptor(String),
    #[error("Invalid file descriptor: {0}")]
    InvalidFileDescriptor(i32),
    #[error("Failed to open file: {0}")]
    FileOpen(#[from] io::Error),
    #[error("Failed to intern headers: {0}")]
    HeaderIntern(#[from] CacheError),
    #[error("Invalid flexible default value: {0}")]
    InvalidFlexibleDefault(String),
    #[error("Invalid null string value: {0}")]
    InvalidNullString(String),
    #[error("Failed to parse CSV record: {0}")]
    CsvParse(#[from] csv::Error),
    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(String),
    #[error("Ruby error: {0}")]
    Ruby(String),
}

impl From<MagnusError> for ReaderError {
    fn from(err: MagnusError) -> Self {
        Self::Ruby(err.to_string())
    }
}

impl From<ReaderError> for MagnusError {
    fn from(err: ReaderError) -> Self {
        let ruby = Ruby::get().unwrap();
        match err {
            ReaderError::CsvParse(csv_err) => {
                if csv_err.to_string().contains("invalid utf-8") {
                    MagnusError::new(ruby.exception_encoding_error(), csv_err.to_string())
                } else {
                    MagnusError::new(ruby.exception_runtime_error(), csv_err.to_string())
                }
            }
            ReaderError::InvalidUtf8(utf8_err) => {
                MagnusError::new(ruby.exception_encoding_error(), utf8_err.to_string())
            }
            _ => MagnusError::new(ruby.exception_runtime_error(), err.to_string()),
        }
    }
}

/// Builder for configuring and creating a RecordReader instance.
///
/// This struct provides a fluent interface for setting up CSV parsing options
/// and creating a RecordReader with the specified configuration.
pub struct RecordReaderBuilder<'a, T: RecordParser<'a>> {
    ruby: Ruby,
    to_read: Value,
    has_headers: bool,
    delimiter: u8,
    quote_char: u8,
    null_string: Option<String>,
    flexible: bool,
    trim: csv::Trim,
    ignore_null_bytes: bool,
    lossy: bool,
    _phantom: PhantomData<T>,
    _phantom_a: PhantomData<&'a ()>,
}

impl<'a, T: RecordParser<'a>> RecordReaderBuilder<'a, T> {
    /// Creates a new builder instance with default settings.
    pub fn new(ruby: Ruby, to_read: Value) -> Self {
        Self {
            ruby,
            to_read,
            has_headers: true,
            delimiter: b',',
            quote_char: b'"',
            null_string: None,
            flexible: false,
            trim: csv::Trim::None,
            ignore_null_bytes: false,
            lossy: false,
            _phantom: PhantomData,
            _phantom_a: PhantomData,
        }
    }

    /// Sets whether the CSV file has headers.
    #[must_use]
    pub fn has_headers(mut self, has_headers: bool) -> Self {
        self.has_headers = has_headers;
        self
    }

    /// Sets the delimiter character for the CSV.
    #[must_use]
    pub fn delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Sets the quote character for the CSV.
    #[must_use]
    pub fn quote_char(mut self, quote_char: u8) -> Self {
        self.quote_char = quote_char;
        self
    }

    /// Sets the string that should be interpreted as null.
    #[must_use]
    pub fn null_string(mut self, null_string: Option<String>) -> Self {
        self.null_string = null_string;
        self
    }

    /// Sets whether the reader should be flexible with field counts.
    #[must_use]
    pub fn flexible(mut self, flexible: bool) -> Self {
        self.flexible = flexible;
        self
    }

    /// Sets the trimming mode for fields.
    #[must_use]
    pub fn trim(mut self, trim: csv::Trim) -> Self {
        self.trim = trim;
        self
    }

    #[must_use]
    pub fn ignore_null_bytes(mut self, ignore_null_bytes: bool) -> Self {
        self.ignore_null_bytes = ignore_null_bytes;
        self
    }

    #[must_use]
    pub fn lossy(mut self, lossy: bool) -> Self {
        self.lossy = lossy;
        self
    }

    /// Handles reading from a file descriptor.
    fn handle_file_descriptor(&self) -> Result<Box<dyn SeekableRead>, ReaderError> {
        let raw_value = self.to_read.as_raw();
        let fd = std::panic::catch_unwind(|| unsafe { rb_sys::rb_io_descriptor(raw_value) })
            .map_err(|e| ReaderError::FileDescriptor(format!("{:?}", e)))?;

        if fd < 0 {
            return Err(ReaderError::InvalidFileDescriptor(fd));
        }

        let file = unsafe { File::from_raw_fd(fd) };
        let forgotten = ForgottenFileHandle(ManuallyDrop::new(file));
        Ok(Box::new(forgotten))
    }

    /// Handles reading from a file path.
    fn handle_file_path(&self) -> Result<Box<dyn SeekableRead>, ReaderError> {
        let path = self.to_read.to_r_string()?.to_string()?;
        let file = File::open(&path)?;

        if path.ends_with(".gz") {
            // For gzipped files, we need to decompress them into memory first
            // since GzDecoder doesn't support seeking
            let mut decoder = GzDecoder::new(BufReader::with_capacity(READ_BUFFER_SIZE, file));
            let mut contents = Vec::new();
            decoder.read_to_end(&mut contents)?;
            Ok(Box::new(std::io::Cursor::new(contents)))
        } else {
            Ok(Box::new(file))
        }
    }

    /// Builds the RecordReader with the configured options.
    pub fn build(self) -> Result<RecordReader<'a, T>, ReaderError> {
        let readable = if self.to_read.is_kind_of(self.ruby.class_io()) {
            self.handle_file_descriptor()?
        } else if self.to_read.is_kind_of(self.ruby.class_string()) {
            self.handle_file_path()?
        } else {
            build_ruby_reader(&self.ruby, self.to_read)?
        };

        let flexible = self.flexible;
        let reader = BufReader::with_capacity(READ_BUFFER_SIZE, readable);

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(self.has_headers)
            .delimiter(self.delimiter)
            .quote(self.quote_char)
            .flexible(flexible)
            .trim(self.trim)
            .from_reader(reader);

        let mut headers =
            RecordReader::<T>::get_headers(&self.ruby, &mut reader, self.has_headers)?;
        if self.ignore_null_bytes {
            headers = headers.iter().map(|h| h.replace("\0", "")).collect();
        }
        let static_headers = StringCache::intern_many(&headers)?;

        let null_string = self
            .null_string
            .map(|s| {
                RString::new(&s)
                    .to_interned_str()
                    .as_str()
                    .map_err(|e| ReaderError::InvalidNullString(format!("{:?}", e)))
            })
            .transpose()?
            .map(Cow::Borrowed);

        Ok(RecordReader::new(
            reader,
            static_headers,
            null_string,
            self.ignore_null_bytes,
            self.lossy,
        ))
    }
}
