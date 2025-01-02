use super::READ_BUFFER_SIZE;
use magnus::{
    value::{Opaque, ReprValue},
    RClass, RString, Ruby, Value,
};
use std::io::{self, Read};
use std::sync::OnceLock;

static STRING_IO_CLASS: OnceLock<Opaque<RClass>> = OnceLock::new();

/// A reader that can handle various Ruby input types (String, StringIO, IO-like objects)
/// and provide a standard Read implementation for them.
pub struct RubyReader<'a, T> {
    #[allow(unused)]
    ruby: &'a Ruby,
    inner: T,
    buffer: Option<Vec<u8>>,
    offset: usize,
    // Number of bytes that have been read into the buffer
    // Used as an upper bound for offset
    buffered_bytes: usize,
}

pub fn build_ruby_reader<'a>(
    ruby: &'a Ruby,
    input: Value,
) -> Result<Box<dyn Read + 'a>, magnus::Error> {
    if RubyReader::is_string_io(ruby, &input) {
        RubyReader::from_string_io(ruby, input)
    } else if RubyReader::is_io_like(&input) {
        RubyReader::from_io(ruby, input)
    } else {
        RubyReader::from_string_like(ruby, input)
    }
}

impl<'a> RubyReader<'a, Value> {
    fn from_io(ruby: &'a Ruby, input: Value) -> Result<Box<dyn Read + 'a>, magnus::Error> {
        if Self::is_io_like(&input) {
            Ok(Box::new(Self::from_io_like(ruby, input)))
        } else {
            Err(magnus::Error::new(
                magnus::exception::type_error(),
                "Input is not an IO-like object",
            ))
        }
    }

    fn is_io_like(input: &Value) -> bool {
        input.respond_to("read", false).unwrap_or(false)
    }

    fn from_io_like(ruby: &'a Ruby, input: Value) -> Self {
        Self {
            ruby,
            inner: input,
            buffer: Some(vec![0; READ_BUFFER_SIZE]),
            offset: 0,
            buffered_bytes: 0,
        }
    }

    fn read_from_buffer(&mut self, to_buf: &mut [u8]) -> Option<io::Result<usize>> {
        if let Some(from_buf) = &self.buffer {
            // If the offset is within the buffered bytes, copy the remaining bytes to the output buffer
            if self.offset < self.buffered_bytes {
                let remaining = self.buffered_bytes - self.offset;
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
            .funcall::<_, _, RString>("read", (buffer.capacity(),))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        if result.is_nil() {
            return Ok(0); // EOF
        }

        let bytes = unsafe { result.as_slice() };

        // Update internal buffer
        let bytes_len = bytes.len();
        if bytes_len == 0 {
            return Ok(0);
        }

        // Only copy what we actually read
        buffer[..bytes_len].copy_from_slice(bytes);
        self.buffered_bytes = bytes_len;

        // Copy to output buffer
        let copy_size = bytes_len.min(buf.len());
        buf[..copy_size].copy_from_slice(&buffer[..copy_size]);
        self.offset = copy_size;
        Ok(copy_size)
    }
}

impl<'a> RubyReader<'a, RString> {
    pub fn from_string_io(
        ruby: &'a Ruby,
        input: Value,
    ) -> Result<Box<dyn Read + 'a>, magnus::Error> {
        if !Self::is_string_io(ruby, &input) {
            return Err(magnus::Error::new(
                magnus::exception::type_error(),
                "Input is not a StringIO",
            ));
        }

        let string_content = input.funcall::<_, _, RString>("string", ()).unwrap();
        Ok(Box::new(Self {
            ruby,
            inner: string_content,
            buffer: None,
            offset: 0,
            buffered_bytes: 0,
        }))
    }

    fn is_string_io(ruby: &Ruby, input: &Value) -> bool {
        let string_io_class = STRING_IO_CLASS.get_or_init(|| {
            let class = RClass::from_value(ruby.eval("StringIO").unwrap()).unwrap();
            Opaque::from(class)
        });
        input.is_kind_of(ruby.get_inner(*string_io_class))
    }

    fn from_string_like(ruby: &'a Ruby, input: Value) -> Result<Box<dyn Read + 'a>, magnus::Error> {
        let string_content = input.funcall::<_, _, RString>("to_str", ()).unwrap();
        Ok(Box::new(Self {
            ruby,
            inner: string_content,
            buffer: None,
            offset: 0,
            buffered_bytes: 0,
        }))
    }
}

impl<'a> Read for RubyReader<'a, Value> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Some(result) = self.read_from_buffer(buf) {
            result
        } else {
            // If the buffer is empty, read from Ruby
            self.read_from_ruby(buf)
        }
    }
}

impl<'a> Read for RubyReader<'a, RString> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let string_buffer = unsafe { self.inner.as_slice() };
        if self.offset >= string_buffer.len() {
            return Ok(0); // EOF
        }

        let remaining = string_buffer.len() - self.offset;
        let copy_size = remaining.min(buf.len());
        buf[..copy_size].copy_from_slice(&string_buffer[self.offset..self.offset + copy_size]);
        self.offset += copy_size;
        Ok(copy_size)
    }
}
