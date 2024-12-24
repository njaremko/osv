use super::{
    header_cache::StringCache,
    parser::RecordParser,
    reader::{ReadImpl, RecordReader},
};
use flate2::read::GzDecoder;
use magnus::{rb_sys::AsRawValue, value::ReprValue, Error, RString, Ruby, Value};
use std::{fs::File, io::Read, marker::PhantomData, os::fd::FromRawFd, thread};

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

    fn get_reader(&self) -> Result<Box<dyn Read + Send + 'static>, Error> {
        let string_io: magnus::RClass = self.ruby.eval("StringIO")?;
        let gzip_reader_class: magnus::RClass = self.ruby.eval("Zlib::GzipReader")?;

        if self.to_read.is_kind_of(string_io) {
            let string: RString = self.to_read.funcall("string", ())?;
            let content = string.to_string()?;
            Ok(Box::new(std::io::Cursor::new(content)))
        } else if self.to_read.is_kind_of(gzip_reader_class) {
            return Err(Error::new(
                self.ruby.exception_runtime_error(),
                "This library does not support GzipReader",
            ));
        } else if self.to_read.is_kind_of(self.ruby.class_io()) {
            let raw_value = self.to_read.as_raw();
            let fd = std::panic::catch_unwind(|| unsafe { rb_sys::rb_io_descriptor(raw_value) })
                .map_err(|e| {
                    Error::new(
                        self.ruby.exception_runtime_error(),
                        format!("Failed to get file descriptor: {:?}", e),
                    )
                })?;

            // Handle invalid file descriptors
            if fd < 0 {
                return Err(Error::new(
                    self.ruby.exception_runtime_error(),
                    "Failed to get file descriptor",
                ));
            }

            let file = unsafe { File::from_raw_fd(fd) };
            Ok(Box::new(file))
        } else {
            let path = self.to_read.to_r_string()?.to_string()?;
            let file = std::fs::File::open(&path).map_err(|e| {
                Error::new(
                    self.ruby.exception_runtime_error(),
                    format!("Failed to open file: {e}"),
                )
            })?;
            if path.ends_with(".gz") {
                let file = GzDecoder::new(file);
                Ok(Box::new(file))
            } else {
                Ok(Box::new(file))
            }
        }
    }

    fn get_single_threaded_reader(&self) -> Result<Box<dyn Read>, Error> {
        let string_io: magnus::RClass = self.ruby.eval("StringIO")?;
        let gzip_reader_class: magnus::RClass = self.ruby.eval("Zlib::GzipReader")?;

        if self.to_read.is_kind_of(string_io) {
            let string: RString = self.to_read.funcall("string", ())?;
            let content = string.to_string()?;
            Ok(Box::new(std::io::Cursor::new(content)))
        } else if self.to_read.is_kind_of(gzip_reader_class) {
            Ok(Box::new(RubyReader::new(self.to_read)))
        } else if self.to_read.is_kind_of(self.ruby.class_io()) {
            let raw_value = self.to_read.as_raw();
            let fd = std::panic::catch_unwind(|| unsafe { rb_sys::rb_io_descriptor(raw_value) })
                .map_err(|e| {
                    Error::new(
                        self.ruby.exception_runtime_error(),
                        format!("Failed to get file descriptor: {:?}", e),
                    )
                })?;

            // Handle invalid file descriptors
            if fd < 0 {
                return Err(Error::new(
                    self.ruby.exception_runtime_error(),
                    "Failed to get file descriptor",
                ));
            }

            println!("fd: {}", fd);

            let file = unsafe { File::from_raw_fd(fd) };
            Ok(Box::new(file))
        } else {
            let path = self.to_read.to_r_string()?.to_string()?;
            let file = std::fs::File::open(&path).map_err(|e| {
                Error::new(
                    self.ruby.exception_runtime_error(),
                    format!("Failed to open file: {e}"),
                )
            })?;
            if path.ends_with(".gz") {
                let file = GzDecoder::new(file);
                Ok(Box::new(file))
            } else {
                Ok(Box::new(file))
            }
        }
    }

    pub fn build(self) -> Result<RecordReader<T>, Error> {
        if let Ok(readable) = self.get_reader() {
            let mut reader = csv::ReaderBuilder::new()
                .has_headers(self.has_headers)
                .delimiter(self.delimiter)
                .quote(self.quote_char)
                .from_reader(readable);

            let headers = RecordReader::<T>::get_headers(self.ruby, &mut reader, self.has_headers)?;
            let null_string = self.null_string;

            let static_headers = StringCache::intern_many(&headers).map_err(|e| {
                Error::new(
                    self.ruby.exception_runtime_error(),
                    format!("Failed to intern headers: {e}"),
                )
            })?;
            let headers_for_cleanup = static_headers.clone();

            let (sender, receiver) = kanal::bounded(self.buffer);
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
        } else {
            let readable = self.get_single_threaded_reader()?;
            let mut reader = csv::ReaderBuilder::new()
                .has_headers(self.has_headers)
                .delimiter(self.delimiter)
                .quote(self.quote_char)
                .from_reader(readable);

            let headers = RecordReader::<T>::get_headers(self.ruby, &mut reader, self.has_headers)?;
            let null_string = self.null_string;

            let static_headers = StringCache::intern_many(&headers).map_err(|e| {
                Error::new(
                    self.ruby.exception_runtime_error(),
                    format!("Failed to intern headers: {e}"),
                )
            })?;
            let headers_for_cleanup = static_headers.clone();

            Ok(RecordReader {
                reader: ReadImpl::SingleThreaded {
                    reader,
                    headers: headers_for_cleanup,
                    null_string,
                },
            })
        }
    }
}

struct RubyReader {
    inner: magnus::Value,
}

impl RubyReader {
    fn new(inner: magnus::Value) -> Self {
        Self { inner }
    }
}

impl std::io::Read for RubyReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let result = self
            .inner
            .funcall::<_, _, magnus::Value>("read", (buf.len(),));
        match result {
            Ok(data) => {
                if data.is_nil() {
                    return Ok(0);
                }

                let bytes = data.to_string().into_bytes();
                let len = bytes.len().min(buf.len());
                buf[..len].copy_from_slice(&bytes[..len]);
                Ok(len)
            }
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Ruby read failed: {}", e),
            )),
        }
    }
}
