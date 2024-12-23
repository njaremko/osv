use super::{
    parser::RecordParser,
    reader::{ReadImpl, RecordReader},
};
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

        if self.to_read.is_kind_of(string_io) {
            let string: RString = self.to_read.funcall("string", ())?;
            let content = string.to_string()?;
            Ok(Box::new(std::io::Cursor::new(content)))
        } else if self.to_read.is_kind_of(self.ruby.class_io()) {
            let fd = unsafe { rb_sys::rb_io_descriptor(self.to_read.as_raw()) };
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
            Ok(Box::new(file))
        }
    }

    pub fn build(self) -> Result<RecordReader<T>, Error> {
        let readable = self.get_reader()?;
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(self.has_headers)
            .delimiter(self.delimiter)
            .quote(self.quote_char)
            .from_reader(readable);

        let headers = RecordReader::<T>::get_headers(self.ruby, &mut reader, self.has_headers)?;
        let null_string = self.null_string;

        let (sender, receiver) = kanal::bounded(self.buffer);
        let handle = thread::spawn(move || {
            let mut record = csv::StringRecord::new();
            while let Ok(true) = reader.read_record(&mut record) {
                let row = T::parse(&headers, &record, &null_string);
                if sender.send(row).is_err() {
                    break;
                }
            }
            let file_to_forget = reader.into_inner();
            std::mem::forget(file_to_forget);
        });

        Ok(RecordReader {
            reader: ReadImpl::MultiThreaded {
                receiver,
                handle: Some(handle),
            },
        })
    }
}
