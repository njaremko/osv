use magnus::{prelude::*, Error, RString, Ruby, Value};
use std::io::Read;

pub struct RubyIOReader {
    io_obj: Value,
}

impl RubyIOReader {
    pub fn new(ruby: &Ruby, value: Value) -> Result<Self, Error> {
        if value.is_kind_of(ruby.class_io()) {
            Ok(RubyIOReader { io_obj: value })
        } else {
            Err(Error::new(
                ruby.exception_runtime_error(),
                "IO object is not a valid IO object",
            ))
        }
    }
}

impl Read for RubyIOReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.io_obj.is_nil() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Cannot read from nil IO object"),
            ));
        }

        let tmp_result: Option<RString> =
            self.io_obj.funcall("read", (buf.len(),)).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to read from IO: {:?}", e),
                )
            })?;

        if let Some(result) = tmp_result {
            // Handle EOF case
            if result.is_nil() {
                return Ok(0);
            }

            let rust_string = result.to_string().map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::Other, "Failed to convert to string")
            })?;
            let bytes = rust_string.as_bytes();

            let bytes_to_copy = rust_string.len().min(buf.len());
            buf[..bytes_to_copy].copy_from_slice(&bytes[..bytes_to_copy]);

            Ok(bytes_to_copy)
        } else {
            return Ok(0);
        }
    }
}
