use super::{
    header_cache::{CacheError, StringCache},
    parser::RecordParser,
    record_reader::{RecordReader, READ_BUFFER_SIZE},
    RubyReader,
};
use flate2::read::GzDecoder;
use magnus::{rb_sys::AsRawValue, value::ReprValue, Error as MagnusError, Ruby, Value};
use std::{
    fs::File,
    io::{self, BufReader, Read},
    marker::PhantomData,
    os::fd::FromRawFd,
};

use thiserror::Error;

pub(crate) static BUFFER_CHANNEL_SIZE: usize = 1024;

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

pub struct RecordReaderBuilder<'a, T: RecordParser + Send + 'a> {
    ruby: &'a Ruby,
    to_read: Value,
    has_headers: bool,
    delimiter: u8,
    quote_char: u8,
    null_string: Option<String>,
    buffer: usize,
    flexible: bool,
    flexible_default: Option<String>,
    trim: csv::Trim,
    _phantom: PhantomData<T>,
}

impl<'a, T: RecordParser + Send + 'static> RecordReaderBuilder<'a, T> {
    pub fn new(ruby: &'a Ruby, to_read: Value) -> Self {
        Self {
            ruby,
            to_read,
            has_headers: true,
            delimiter: b',',
            quote_char: b'"',
            null_string: None,
            buffer: BUFFER_CHANNEL_SIZE,
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

    pub fn buffer(mut self, buffer: usize) -> Self {
        self.buffer = buffer;
        self
    }

    pub fn flexible(mut self, flexible: bool) -> Self {
        self.flexible = flexible;
        self
    }

    pub fn flexible_default(mut self, flexible_default: Option<String>) -> Self {
        self.flexible_default = flexible_default;
        self
    }

    pub fn trim(mut self, trim: csv::Trim) -> Self {
        self.trim = trim;
        self
    }

    fn handle_file_descriptor(&self) -> Result<Box<dyn Read + Send + 'static>, ReaderError> {
        let raw_value = self.to_read.as_raw();
        let fd = std::panic::catch_unwind(|| unsafe { rb_sys::rb_io_descriptor(raw_value) })
            .map_err(|_| {
                ReaderError::FileDescriptor("Failed to get file descriptor".to_string())
            })?;

        if fd < 0 {
            return Err(ReaderError::InvalidFileDescriptor);
        }

        let file = unsafe { File::from_raw_fd(fd) };
        Ok(Box::new(BufReader::with_capacity(READ_BUFFER_SIZE, file)))
    }

    fn handle_file_path(&self) -> Result<Box<dyn Read + Send + 'static>, ReaderError> {
        let path = self.to_read.to_r_string()?.to_string()?;
        let file = File::open(&path)?;

        Ok(if path.ends_with(".gz") {
            Box::new(GzDecoder::new(BufReader::with_capacity(
                READ_BUFFER_SIZE,
                file,
            )))
        } else {
            Box::new(BufReader::with_capacity(READ_BUFFER_SIZE, file))
        })
    }

    pub fn build(self) -> Result<RecordReader<'a, T>, ReaderError> {
        if self.to_read.is_kind_of(self.ruby.class_io()) {
            let readable = self.handle_file_descriptor()?;
            self.build_multi_threaded(readable, true)
        } else if self.to_read.is_kind_of(self.ruby.class_string()) {
            let readable = self.handle_file_path()?;
            self.build_multi_threaded(readable, false)
        } else {
            let readable: Box<dyn Read> =
                Box::new(RubyReader::from_string_io(self.ruby, self.to_read))
                    .unwrap_or_else(|_| Box::new(RubyReader::from_value(self.ruby, self.to_read)));

            self.build_single_threaded(readable)
        }
    }

    fn build_multi_threaded(
        self,
        readable: Box<dyn Read + Send + 'static>,
        should_forget: bool,
    ) -> Result<RecordReader<'static, T>, ReaderError> {
        let flexible = self.flexible || self.flexible_default.is_some();
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(self.has_headers)
            .delimiter(self.delimiter)
            .quote(self.quote_char)
            .flexible(flexible)
            .trim(self.trim)
            .from_reader(readable);

        let headers = RecordReader::<T>::get_headers(self.ruby, &mut reader, self.has_headers)?;
        let static_headers = StringCache::intern_many(&headers)?;

        Ok(RecordReader::new_multi_threaded(
            reader,
            static_headers,
            self.buffer,
            self.null_string,
            self.flexible_default,
            should_forget,
        ))
    }

    fn build_single_threaded(
        self,
        readable: Box<dyn Read + 'a>,
    ) -> Result<RecordReader<'a, T>, ReaderError> {
        let flexible = self.flexible || self.flexible_default.is_some();
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(self.has_headers)
            .delimiter(self.delimiter)
            .quote(self.quote_char)
            .flexible(flexible)
            .trim(self.trim)
            .from_reader(readable);

        let headers = RecordReader::<T>::get_headers(self.ruby, &mut reader, self.has_headers)?;
        let static_headers = StringCache::intern_many(&headers)?;

        Ok(RecordReader::new_single_threaded(
            reader,
            static_headers,
            self.null_string,
            self.flexible_default,
        ))
    }
}
