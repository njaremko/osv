use super::READ_BUFFER_SIZE;
use crate::utils::string_to_buffer;
use magnus::{
    rb_sys::AsRawValue,
    value::{Opaque, ReprValue},
    Error as MagnusError, RClass, Ruby, Value,
};
use std::io::{self, Read};
use std::sync::OnceLock;

static STRING_IO_CLASS: OnceLock<Opaque<RClass>> = OnceLock::new();

/// A reader that can handle various Ruby input types (String, StringIO, IO-like objects)
/// and provide a standard Read implementation for them.
pub struct RubyReader<'a> {
    #[allow(unused)]
    ruby: &'a Ruby,
    inner: Value,
    buffer: Option<Vec<u8>>,
    string_buffer: Option<&'a [u8]>,
    offset: usize,
}

impl<'a> RubyReader<'a> {
    /// Creates a new RubyReader from a Ruby value.
    ///
    /// The input can be:
    /// - A StringIO object
    /// - An IO-like object that responds to `read`
    /// - A String-like object that responds to `to_str`
    pub fn new(ruby: &'a Ruby, input: Value) -> Self {
        if Self::is_string_io(ruby, &input) {
            return Self::from_string_io(ruby, input);
        }

        if Self::is_io_like(&input) {
            return Self::from_io_like(ruby, input);
        }

        Self::from_string_like(ruby, input)
    }

    fn is_string_io(ruby: &Ruby, input: &Value) -> bool {
        let string_io_class = STRING_IO_CLASS.get_or_init(|| {
            let class = RClass::from_value(ruby.eval("StringIO").unwrap()).unwrap();
            Opaque::from(class)
        });
        input.is_kind_of(ruby.get_inner(*string_io_class))
    }

    fn is_io_like(input: &Value) -> bool {
        input.respond_to("read", false).unwrap_or(false)
    }

    fn from_string_io(ruby: &'a Ruby, input: Value) -> Self {
        let string_content = input.funcall::<_, _, Value>("string", ()).unwrap();
        Self {
            ruby,
            inner: string_content,
            buffer: None,
            offset: 0,
            string_buffer: Some(string_to_buffer(string_content)),
        }
    }

    fn from_io_like(ruby: &'a Ruby, input: Value) -> Self {
        Self {
            ruby,
            inner: input,
            buffer: Some(Vec::with_capacity(READ_BUFFER_SIZE)),
            offset: 0,
            string_buffer: None,
        }
    }

    fn from_string_like(ruby: &'a Ruby, input: Value) -> Self {
        let string_content = input.funcall::<_, _, Value>("to_str", ()).unwrap();
        Self {
            ruby,
            inner: string_content,
            buffer: None,
            offset: 0,
            string_buffer: Some(string_to_buffer(string_content)),
        }
    }

    fn read_from_string_buffer(&mut self, from_buf: &[u8], to_buf: &mut [u8]) -> io::Result<usize> {
        let string_buffer = from_buf;
        if self.offset >= string_buffer.len() {
            return Ok(0); // EOF
        }

        let remaining = string_buffer.len() - self.offset;
        let copy_size = remaining.min(to_buf.len());
        to_buf[..copy_size].copy_from_slice(&string_buffer[self.offset..self.offset + copy_size]);
        self.offset += copy_size;
        Ok(copy_size)
    }

    fn read_from_buffer(&mut self, to_buf: &mut [u8]) -> Option<io::Result<usize>> {
        if let Some(from_buf) = &self.buffer {
            if self.offset < from_buf.len() {
                let remaining = from_buf.len() - self.offset;
                let copy_size = remaining.min(to_buf.len());
                to_buf[..copy_size]
                    .copy_from_slice(&from_buf[self.offset..self.offset + copy_size]);
                self.offset += copy_size;
                Some(Ok(copy_size))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn read_from_ruby(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let buffer = self.buffer.as_mut().unwrap();
        let result = self
            .inner
            .funcall::<_, _, Value>("read", (buffer.capacity(),))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        if result.is_nil() {
            return Ok(0); // EOF
        }

        let bytes = unsafe {
            let raw = result.as_raw();
            std::slice::from_raw_parts(
                rb_sys::RSTRING_PTR(raw) as *const u8,
                rb_sys::RSTRING_LEN(raw) as usize,
            )
        };

        // Update internal buffer
        if bytes.len() == buffer.len() {
            buffer.copy_from_slice(bytes);
        } else {
            buffer.clear();
            buffer.extend_from_slice(bytes);
        }
        self.offset = 0;

        // Copy to output buffer
        let copy_size = buffer.len().min(buf.len());
        buf[..copy_size].copy_from_slice(&buffer[..copy_size]);
        self.offset = copy_size;
        Ok(copy_size)
    }
}

impl<'a> Read for RubyReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Some(string_buffer) = self.string_buffer {
            return self.read_from_string_buffer(string_buffer, buf);
        }

        if let Some(result) = self.read_from_buffer(buf) {
            return result;
        }

        self.read_from_ruby(buf)
    }
}
