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
}

/// Parse common arguments for CSV parsing
pub fn parse_csv_args(args: &[Value]) -> Result<CsvArgs, Error> {
    let parsed_args = scan_args::<(Value,), (), (), (), _, ()>(args)?;
    let (to_read,) = parsed_args.required;

    let kwargs =
        get_kwargs::<_, (), (Option<bool>, Option<String>, Option<String>, Option<String>), ()>(
            parsed_args.keywords,
            &[],
            &["has_headers", "col_sep", "quote_char", "null_string"],
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

    Ok(CsvArgs {
        to_read,
        has_headers,
        delimiter,
        quote_char,
        null_string,
    })
}
