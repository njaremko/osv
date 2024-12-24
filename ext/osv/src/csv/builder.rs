use super::{
    header_cache::{CacheError, StringCache},
    parser::RecordParser,
    read_impl::ReadImpl,
    reader::RecordReader,
};
use flate2::read::GzDecoder;
use magnus::{rb_sys::AsRawValue, value::ReprValue, Error as MagnusError, RString, Ruby, Value};
use std::{
    fs::File,
    io::{self, Read},
    marker::PhantomData,
    os::fd::FromRawFd,
    thread,
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
    #[error("Unsupported GzipReader")]
    UnsupportedGzipReader,
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

pub struct RecordReaderBuilder<'a, T: RecordParser + Send + 'static> {
    ruby: &'a Ruby,
    to_read: Value,
    has_headers: bool,
    delimiter: u8,
    quote_char: u8,
    null_string: String,
    buffer: usize,
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
            null_string: String::new(),
            buffer: 1000,
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

    pub fn null_string(mut self, null_string: String) -> Self {
        self.null_string = null_string;
        self
    }

    pub fn buffer(mut self, buffer: usize) -> Self {
        self.buffer = buffer;
        self
    }

    fn handle_string_io(&self) -> Result<Box<dyn Read + Send + 'static>, ReaderError> {
        let string: RString = self.to_read.funcall("string", ())?;
        let content = string.to_string()?;
        Ok(Box::new(std::io::Cursor::new(content)))
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
        Ok(Box::new(file))
    }

    fn handle_file_path(&self) -> Result<Box<dyn Read + Send + 'static>, ReaderError> {
        let path = self.to_read.to_r_string()?.to_string()?;
        let file = File::open(&path)?;

        Ok(if path.ends_with(".gz") {
            Box::new(GzDecoder::new(file))
        } else {
            Box::new(file)
        })
    }

    fn get_reader(&self) -> Result<Box<dyn Read + Send + 'static>, ReaderError> {
        let string_io: magnus::RClass = self.ruby.eval("StringIO")?;
        let gzip_reader_class: magnus::RClass = self.ruby.eval("Zlib::GzipReader")?;

        if self.to_read.is_kind_of(string_io) {
            self.handle_string_io()
        } else if self.to_read.is_kind_of(gzip_reader_class) {
            Err(ReaderError::UnsupportedGzipReader)
        } else if self.to_read.is_kind_of(self.ruby.class_io()) {
            self.handle_file_descriptor()
        } else {
            self.handle_file_path()
        }
    }

    fn get_single_threaded_reader(&self) -> Result<Box<dyn Read>, ReaderError> {
        let string_io: magnus::RClass = self.ruby.eval("StringIO")?;
        let gzip_reader_class: magnus::RClass = self.ruby.eval("Zlib::GzipReader")?;

        if self.to_read.is_kind_of(string_io) {
            self.handle_string_io().map(|r| -> Box<dyn Read> { r })
        } else if self.to_read.is_kind_of(gzip_reader_class) {
            Ok(Box::new(RubyReader::new(self.to_read)))
        } else if self.to_read.is_kind_of(self.ruby.class_io()) {
            self.handle_file_descriptor()
                .map(|r| -> Box<dyn Read> { r })
        } else {
            self.handle_file_path().map(|r| -> Box<dyn Read> { r })
        }
    }

    pub fn build(self) -> Result<RecordReader<T>, ReaderError> {
        match self.get_reader() {
            Ok(readable) => self.build_multi_threaded(readable),
            Err(_) => {
                let readable = self.get_single_threaded_reader()?;
                self.build_single_threaded(readable)
            }
        }
    }

    fn build_multi_threaded(
        self,
        readable: Box<dyn Read + Send + 'static>,
    ) -> Result<RecordReader<T>, ReaderError> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(self.has_headers)
            .delimiter(self.delimiter)
            .quote(self.quote_char)
            .from_reader(readable);

        let headers = RecordReader::<T>::get_headers(self.ruby, &mut reader, self.has_headers)?;
        let static_headers = StringCache::intern_many(&headers)?;
        let headers_for_cleanup = static_headers.clone();

        let (sender, receiver) = kanal::bounded(self.buffer);
        let null_string = self.null_string.clone();

        let handle = thread::spawn(move || {
            let mut record = csv::StringRecord::new();
            while let Ok(true) = reader.read_record(&mut record) {
                let row = T::parse(&static_headers, &record, &null_string);
                if sender.send(row).is_err() {
                    break;
                }
            }
            let file_to_forget = reader.into_inner();
            std::mem::forget(file_to_forget);
        });

        Ok(RecordReader {
            reader: ReadImpl::MultiThreaded {
                headers: headers_for_cleanup,
                receiver,
                handle: Some(handle),
            },
        })
    }

    fn build_single_threaded(
        self,
        readable: Box<dyn Read>,
    ) -> Result<RecordReader<T>, ReaderError> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(self.has_headers)
            .delimiter(self.delimiter)
            .quote(self.quote_char)
            .from_reader(readable);

        let headers = RecordReader::<T>::get_headers(self.ruby, &mut reader, self.has_headers)?;
        let static_headers = StringCache::intern_many(&headers)?;

        Ok(RecordReader {
            reader: ReadImpl::SingleThreaded {
                reader,
                headers: static_headers,
                null_string: self.null_string,
            },
        })
    }
}

struct RubyReader {
    inner: Value,
}

impl RubyReader {
    fn new(inner: Value) -> Self {
        Self { inner }
    }
}

impl Read for RubyReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let result = self.inner.funcall::<_, _, Value>("read", (buf.len(),));
        match result {
            Ok(data) => {
                if data.is_nil() {
                    return Ok(0);
                }

                let string = RString::from_value(data).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::Other, "Failed to convert to RString")
                })?;
                let bytes = unsafe { string.as_slice() };
                let len = bytes.len().min(buf.len());
                buf[..len].copy_from_slice(&bytes[..len]);
                Ok(len)
            }
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
        }
    }
}
