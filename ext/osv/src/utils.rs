use magnus::{
    prelude::*,
    scan_args::{get_kwargs, scan_args},
    Error, RString, Ruby, Value,
};
use std::io::Read;

use crate::io_reader::RubyIOReader;

/// Helper function to get a readable from either an IO object or a file path
pub fn get_readable(ruby: &Ruby, to_read: Value) -> Result<Box<dyn Read>, Error> {
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
pub fn create_csv_reader(
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
pub fn setup_csv_parser(
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
pub fn parse_csv_args(args: &[Value]) -> Result<(Value, bool, Option<String>), Error> {
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
