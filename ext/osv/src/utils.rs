use magnus::{
    scan_args::{get_kwargs, scan_args},
    Error, Value,
};

#[derive(Debug)]
pub struct CsvArgs {
    pub to_read: Value,
    pub has_headers: bool,
    pub delimiter: u8,
    pub quote_char: u8,
    pub null_string: String,
    pub buffer_size: usize,
    pub result_type: String,
}

/// Parse common arguments for CSV parsing
pub fn parse_csv_args(args: &[Value]) -> Result<CsvArgs, Error> {
    let parsed_args = scan_args::<(Value,), (), (), (), _, ()>(args)?;
    let (to_read,) = parsed_args.required;

    let kwargs = get_kwargs::<
        _,
        (),
        (
            Option<bool>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<usize>,
            Option<String>,
        ),
        (),
    >(
        parsed_args.keywords,
        &[],
        &[
            "has_headers",
            "col_sep",
            "quote_char",
            "nil_string",
            "buffer_size",
            "result_type",
        ],
    )?;

    let has_headers = kwargs.optional.0.unwrap_or(true);

    let delimiter = *kwargs
        .optional
        .1
        .unwrap_or_else(|| ",".to_string())
        .as_bytes()
        .first()
        .ok_or_else(|| {
            Error::new(
                magnus::exception::runtime_error(),
                "Delimiter cannot be empty",
            )
        })?;

    let quote_char = *kwargs
        .optional
        .2
        .unwrap_or_else(|| "\"".to_string())
        .as_bytes()
        .first()
        .ok_or_else(|| {
            Error::new(
                magnus::exception::runtime_error(),
                "Quote character cannot be empty",
            )
        })?;

    let null_string = kwargs.optional.3.unwrap_or_else(|| "".to_string());

    let buffer_size = kwargs.optional.4.unwrap_or(1000);

    let result_type = {
        let rt = kwargs.optional.5.unwrap_or_else(|| "hash".to_string());
        if rt != "hash" && rt != "array" {
            return Err(Error::new(
                magnus::exception::runtime_error(),
                "result_type must be either 'hash' or 'array'",
            ));
        }
        rt
    };

    Ok(CsvArgs {
        to_read,
        has_headers,
        delimiter,
        quote_char,
        null_string,
        buffer_size,
        result_type,
    })
}
