use flate2::bufread::GzDecoder;
use magnus::{
    value::{Opaque, ReprValue},
    RString, Ruby, Value,
};
use std::{
    fs::File,
    io::{self, BufReader, Read, Write},
};

use super::{builder::ReaderError, record_reader::READ_BUFFER_SIZE};

/// A reader that can handle various Ruby input types (String, StringIO, IO-like objects)
/// and provide a standard Read implementation for them.
pub enum RubyReader {
    String {
        inner: Opaque<RString>,
        offset: usize,
    },
    RubyIoLike {
        inner: Opaque<Value>,
    },
    NativeProxyIoLike {
        proxy_file: Box<dyn Read>,
    },
}

impl RubyReader {
    fn is_io_like(value: &Value) -> bool {
        value.respond_to("read", false).unwrap_or(false)
    }
}

impl TryFrom<Value> for RubyReader {
    type Error = ReaderError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let ruby = unsafe { Ruby::get_unchecked() };
        if RubyReader::is_io_like(&value) {
            Ok(RubyReader::RubyIoLike {
                inner: Opaque::from(value),
            })
        } else if value.is_kind_of(ruby.class_string()) {
            let ruby_string = value.to_r_string()?;
            let file_path = ruby_string.to_string()?;
            let file = File::open(&file_path)?;

            let x: Box<dyn Read> = if file_path.ends_with(".gz") {
                let decoder = GzDecoder::new(BufReader::with_capacity(READ_BUFFER_SIZE, file));
                Box::new(decoder)
            } else {
                Box::new(file)
            };

            Ok(RubyReader::NativeProxyIoLike { proxy_file: x })
        } else {
            // Try calling `to_str`, and if that fails, try `to_s`
            let string_content = value
                .funcall::<_, _, RString>("to_str", ())
                .or_else(|_| value.funcall::<_, _, RString>("to_s", ()))?;
            Ok(RubyReader::String {
                inner: Opaque::from(string_content),
                offset: 0,
            })
        }
    }
}

impl Read for RubyReader {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let ruby = unsafe { Ruby::get_unchecked() };
        match self {
            RubyReader::NativeProxyIoLike { proxy_file } => proxy_file.read(buf),
            RubyReader::String { inner, offset } => {
                let unwrapped_inner = ruby.get_inner(*inner);

                let string_buffer = unsafe { unwrapped_inner.as_slice() };
                if *offset >= string_buffer.len() {
                    return Ok(0); // EOF
                }

                let remaining = string_buffer.len() - *offset;
                let copy_size = remaining.min(buf.len());
                buf[..copy_size].copy_from_slice(&string_buffer[*offset..*offset + copy_size]);

                *offset += copy_size;

                Ok(copy_size)
            }
            RubyReader::RubyIoLike { inner } => {
                let unwrapped_inner = ruby.get_inner(*inner);

                let bytes = unwrapped_inner
                    .funcall::<_, _, Option<RString>>("read", (buf.len(),))
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

                match bytes {
                    Some(bytes) => {
                        let string_buffer = unsafe { bytes.as_slice() };
                        buf.write_all(string_buffer)?;
                        Ok(string_buffer.len())
                    }
                    None => Ok(0),
                }
            }
        }
    }
}
