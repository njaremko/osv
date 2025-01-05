use super::{
    header_cache::{CacheError, StringCache},
    parser::RecordParser,
    record_reader::{RecordReader, READ_BUFFER_SIZE},
    ruby_reader::{build_ruby_reader, SeekableRead},
    ForgottenFileHandle,
};
use flate2::read::GzDecoder;
use magnus::{rb_sys::AsRawValue, value::ReprValue, Error as MagnusError, Ruby, Value};
use std::{
    fs::File,
    io::{self, BufReader, Read},
    marker::PhantomData,
    mem::ManuallyDrop,
    os::fd::FromRawFd,
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReaderError {
    #[error("Failed to get file descriptor: {0}")]
    FileDescriptor(String),
    #[error("Invalid file descriptor")]
    InvalidFileDescriptor,
    #[error("Failed to open file: {0}")]
    FileOpen(#[from] io::Error),
    #[error("Failed to intern headers: {0}")]
    HeaderIntern(#[from] CacheError),
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
        MagnusError::new(
            Ruby::get().unwrap().exception_runtime_error(),
            err.to_string(),
        )
    }
}

pub struct RecordReaderBuilder<'a, T: RecordParser<'a>> {
    ruby: &'a Ruby,
    to_read: Value,
    has_headers: bool,
    delimiter: u8,
    quote_char: u8,
    null_string: Option<String>,
    flexible: bool,
    flexible_default: Option<&'a str>,
    trim: csv::Trim,
    _phantom: PhantomData<T>,
}

impl<'a, T: RecordParser<'a>> RecordReaderBuilder<'a, T> {
    pub fn new(ruby: &'a Ruby, to_read: Value) -> Self {
        Self {
            ruby,
            to_read,
            has_headers: true,
            delimiter: b',',
            quote_char: b'"',
            null_string: None,
            flexible: false,
            flexible_default: None,
            trim: csv::Trim::None,
            _phantom: PhantomData,
        }
    }

    pub fn has_headers(mut self, has_headers: bool) -> Self {
        self.has_headers = has_headers;
        self
    }

    pub fn delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn quote_char(mut self, quote_char: u8) -> Self {
        self.quote_char = quote_char;
        self
    }

    pub fn null_string(mut self, null_string: Option<String>) -> Self {
        self.null_string = null_string;
        self
    }

    pub fn flexible(mut self, flexible: bool) -> Self {
        self.flexible = flexible;
        self
    }

    pub fn flexible_default(mut self, flexible_default: Option<&'a str>) -> Self {
        self.flexible_default = flexible_default;
        self
    }

    pub fn trim(mut self, trim: csv::Trim) -> Self {
        self.trim = trim;
        self
    }

    fn handle_file_descriptor(&self) -> Result<Box<dyn SeekableRead>, ReaderError> {
        let raw_value = self.to_read.as_raw();
        let fd = std::panic::catch_unwind(|| unsafe { rb_sys::rb_io_descriptor(raw_value) })
            .map_err(|_| {
                ReaderError::FileDescriptor("Failed to get file descriptor".to_string())
            })?;

        if fd < 0 {
            return Err(ReaderError::InvalidFileDescriptor);
        }

        let file = unsafe { File::from_raw_fd(fd) };
        let forgotten = ForgottenFileHandle(ManuallyDrop::new(file));
        Ok(Box::new(BufReader::with_capacity(
            READ_BUFFER_SIZE,
            forgotten,
        )))
    }

    fn handle_file_path(&self) -> Result<Box<dyn SeekableRead>, ReaderError> {
        let path = self.to_read.to_r_string()?.to_string()?;
        let file = File::open(&path)?;

        if path.ends_with(".gz") {
            // For gzipped files, we need to decompress them into memory first
            // since GzDecoder doesn't support seeking
            let mut decoder = GzDecoder::new(BufReader::with_capacity(READ_BUFFER_SIZE, file));
            let mut contents = Vec::new();
            decoder.read_to_end(&mut contents)?;
            let cursor = std::io::Cursor::new(contents);
            Ok(Box::new(BufReader::new(cursor)))
        } else {
            Ok(Box::new(BufReader::with_capacity(READ_BUFFER_SIZE, file)))
        }
    }

    pub fn build(self) -> Result<RecordReader<'a, T>, ReaderError> {
        let readable = if self.to_read.is_kind_of(self.ruby.class_io()) {
            self.handle_file_descriptor()?
        } else if self.to_read.is_kind_of(self.ruby.class_string()) {
            self.handle_file_path()?
        } else {
            build_ruby_reader(self.ruby, self.to_read)?
        };

        let buffered_reader = BufReader::with_capacity(READ_BUFFER_SIZE, readable);
        let flexible = self.flexible || self.flexible_default.is_some();

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(self.has_headers)
            .delimiter(self.delimiter)
            .quote(self.quote_char)
            .flexible(flexible)
            .trim(self.trim)
            .from_reader(buffered_reader);

        let headers = RecordReader::<T>::get_headers(self.ruby, &mut reader, self.has_headers)?;
        let static_headers = StringCache::intern_many(&headers)?;

        Ok(RecordReader::new(
            reader,
            static_headers,
            self.null_string,
            self.flexible_default,
        ))
    }
}
