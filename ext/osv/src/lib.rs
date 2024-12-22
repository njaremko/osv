use magnus::{
    block::Yield,
    prelude::*,
    scan_args::{get_kwargs, scan_args},
    Error, RString, Ruby, Value,
};
use std::{collections::VecDeque, io::Read};

/// Initializes the Ruby extension and defines methods.
#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("OSV")?;
    module.define_module_function("for_each", magnus::method!(parse_csv, -1))?;
    module.define_module_function("for_each_compat", magnus::method!(parse_compat, -1))?;
    Ok(())
}

/// Helper function to get a readable from either an IO object or a file path
fn get_readable(ruby: &Ruby, to_read: Value) -> Result<Box<dyn Read>, Error> {
    if to_read.is_kind_of(ruby.class_io()) {
        let reader = RubyIOReader::new(ruby, to_read)?;
        Ok(Box::new(reader))
    } else {
        let path = to_read.to_r_string()?.to_string()?;
        let file = std::fs::File::open(&path).map_err(|e| {
            Error::new(
                ruby.exception_runtime_error(),
                format!("Failed to open file: {}", e),
            )
        })?;
        Ok(Box::new(file))
    }
}

/// Helper function to create a CSV reader with the given configuration
fn create_csv_reader(
    ruby: &Ruby,
    to_read: Value,
    has_headers: bool,
    delimiter: Option<String>,
) -> Result<csv::Reader<Box<dyn Read>>, Error> {
    let readable = get_readable(ruby, to_read)?;
    let delimiter = delimiter.unwrap_or_else(|| ",".to_string());

    let rdr = csv::ReaderBuilder::new()
        .has_headers(has_headers)
        .delimiter(delimiter.as_bytes()[0])
        .from_reader(readable);

    Ok(rdr)
}

/// Common setup for CSV parsing, returns the reader and headers
fn setup_csv_parser(
    ruby: &Ruby,
    to_read: Value,
    has_headers: bool,
    delimiter: Option<String>,
) -> Result<(csv::Reader<Box<dyn Read>>, Vec<String>), Error> {
    let mut rdr = create_csv_reader(ruby, to_read, has_headers, delimiter)?;

    let first_row = rdr.headers().unwrap().clone();
    let num_fields = first_row.len();

    let headers = if has_headers {
        first_row.iter().map(|h| h.to_string()).collect()
    } else {
        (0..num_fields).map(|i| format!("c{}", i)).collect()
    };

    Ok((rdr, headers))
}

/// Parse common arguments for CSV parsing
fn parse_csv_args(args: &[Value]) -> Result<(Value, bool, Option<String>), Error> {
    let parsed_args = scan_args::<(Value,), (), (), (), _, ()>(args)?;
    let (to_read,) = parsed_args.required;

    let kwargs = get_kwargs::<_, (), (Option<bool>, Option<String>), ()>(
        parsed_args.keywords,
        &[],
        &["has_headers", "delimiter"],
    )?;

    let has_headers = kwargs.optional.0.unwrap_or(true);

    Ok((to_read, has_headers, kwargs.optional.1))
}

/// Parses CSV data from a file and yields each row as a hash to the block.
fn parse_csv(
    ruby: &Ruby,
    rb_self: Value,
    args: &[Value],
) -> Result<Yield<impl Iterator<Item = std::collections::HashMap<String, String>>>, Error> {
    if !ruby.block_given() {
        return Ok(Yield::Enumerator(rb_self.enumeratorize("for_each", args)));
    }

    let (to_read, has_headers, delimiter) = parse_csv_args(args)?;
    let (rdr, headers) = setup_csv_parser(ruby, to_read, has_headers, delimiter)?;

    let iter = BufferedRecordsAsHash {
        reader: rdr,
        buffer: VecDeque::with_capacity(1000),
        record: csv::StringRecord::new(),
        headers: headers,
    };

    Ok(Yield::Iter(iter))
}

fn parse_compat(
    ruby: &Ruby,
    rb_self: Value,
    args: &[Value],
) -> Result<Yield<impl Iterator<Item = Vec<String>>>, Error> {
    if !ruby.block_given() {
        return Ok(Yield::Enumerator(
            rb_self.enumeratorize("for_each_compat", args),
        ));
    }

    let (to_read, has_headers, delimiter) = parse_csv_args(args)?;
    let (rdr, _) = setup_csv_parser(ruby, to_read, has_headers, delimiter)?;

    let iter = BufferedRecords {
        reader: rdr,
        buffer: VecDeque::with_capacity(1000),
        record: csv::StringRecord::new(),
    };

    Ok(Yield::Iter(iter))
}

struct BufferedRecords {
    reader: csv::Reader<Box<dyn Read>>,
    buffer: VecDeque<Vec<String>>,
    record: csv::StringRecord,
}

impl Iterator for BufferedRecords {
    type Item = Vec<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            // Refill buffer with up to 1000 records
            while self.buffer.len() < 1000 {
                if !self.reader.read_record(&mut self.record).unwrap() {
                    break;
                }
                let row = self.record.iter().map(|field| field.to_string()).collect();
                self.buffer.push_back(row);
            }

            if self.buffer.is_empty() {
                return None;
            }
        }

        self.buffer.pop_front()
    }
}

struct BufferedRecordsAsHash {
    reader: csv::Reader<Box<dyn Read>>,
    buffer: VecDeque<std::collections::HashMap<String, String>>,
    record: csv::StringRecord,
    headers: Vec<String>,
}

impl Iterator for BufferedRecordsAsHash {
    type Item = std::collections::HashMap<String, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            // Refill buffer with up to 1000 records
            while self.buffer.len() < 1000 {
                if !self.reader.read_record(&mut self.record).unwrap() {
                    break;
                }
                let mut map = std::collections::HashMap::new();
                for (i, field) in self.record.iter().enumerate() {
                    let header = if i < self.headers.len() {
                        self.headers[i].to_string()
                    } else {
                        format!("c{}", i)
                    };
                    map.insert(header, field.to_string());
                }
                self.buffer.push_back(map);
            }

            if self.buffer.is_empty() {
                return None;
            }
        }

        self.buffer.pop_front()
    }
}

struct RubyIOReader {
    io_obj: Value,
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

impl RubyIOReader {
    fn new(ruby: &Ruby, value: Value) -> Result<Self, Error> {
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
