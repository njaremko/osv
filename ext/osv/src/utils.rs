use magnus::{
    scan_args::{get_kwargs, scan_args},
    value::ReprValue,
    Error, RString, Ruby, Symbol, Value,
};

use crate::csv::BUFFER_CHANNEL_SIZE;

fn parse_string_or_symbol(ruby: &Ruby, value: Value) -> Result<Option<String>, Error> {
    if value.is_nil() {
        Ok(None)
    } else if value.is_kind_of(ruby.class_string()) {
        RString::from_value(value)
            .ok_or_else(|| Error::new(magnus::exception::type_error(), "Invalid string value"))?
            .to_string()
            .map(|s| Some(s))
    } else if value.is_kind_of(ruby.class_symbol()) {
        Symbol::from_value(value)
            .ok_or_else(|| Error::new(magnus::exception::type_error(), "Invalid symbol value"))?
            .funcall("to_s", ())
            .map(|s| Some(s))
    } else {
        Err(Error::new(
            magnus::exception::type_error(),
            "Value must be a String or Symbol",
        ))
    }
}

#[derive(Debug)]
pub struct CsvArgs {
    pub to_read: Value,
    pub has_headers: bool,
    pub delimiter: u8,
    pub quote_char: u8,
    pub null_string: Option<String>,
    pub buffer_size: usize,
    pub result_type: String,
    pub flexible: bool,
    pub flexible_default: Option<String>,
    pub trim: csv::Trim,
}

/// Parse common arguments for CSV parsing
pub fn parse_csv_args(ruby: &Ruby, args: &[Value]) -> Result<CsvArgs, Error> {
    let parsed_args = scan_args::<(Value,), (), (), (), _, ()>(args)?;
    let (to_read,) = parsed_args.required;

    let kwargs = get_kwargs::<
        _,
        (),
        (
            Option<bool>,
            Option<String>,
            Option<String>,
            Option<Option<String>>,
            Option<usize>,
            Option<Value>,
            Option<bool>,
            Option<Option<String>>,
            Option<Value>,
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
            "flexible",
            "flexible_default",
            "trim",
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

    let null_string = kwargs.optional.3.unwrap_or_default();

    let buffer_size = kwargs.optional.4.unwrap_or(BUFFER_CHANNEL_SIZE);

    let result_type = match kwargs
        .optional
        .5
        .map(|value| parse_string_or_symbol(ruby, value))
    {
        Some(Ok(Some(parsed))) => match parsed.as_str() {
            "hash" | "array" => parsed,
            _ => {
                return Err(Error::new(
                    magnus::exception::runtime_error(),
                    "result_type must be either 'hash' or 'array'",
                ))
            }
        },
        Some(Ok(None)) => String::from("hash"),
        Some(Err(_)) => {
            return Err(Error::new(
                magnus::exception::type_error(),
                "result_type must be a String or Symbol",
            ))
        }
        None => String::from("hash"),
    };

    let flexible = kwargs.optional.6.unwrap_or_default();

    let flexible_default = kwargs.optional.7.unwrap_or_default();

    let trim = match kwargs
        .optional
        .8
        .map(|value| parse_string_or_symbol(ruby, value))
    {
        Some(Ok(Some(parsed))) => match parsed.as_str() {
            "all" => csv::Trim::All,
            "headers" => csv::Trim::Headers,
            "fields" => csv::Trim::Fields,
            invalid => {
                return Err(Error::new(
                    magnus::exception::runtime_error(),
                    format!(
                        "trim must be either 'all', 'headers', or 'fields' but got '{}'",
                        invalid
                    ),
                ))
            }
        },
        Some(Ok(None)) => csv::Trim::None,
        Some(Err(_)) => {
            return Err(Error::new(
                magnus::exception::type_error(),
                "trim must be a String or Symbol",
            ))
        }
        None => csv::Trim::None,
    };

    Ok(CsvArgs {
        to_read,
        has_headers,
        delimiter,
        quote_char,
        null_string,
        buffer_size,
        result_type,
        flexible,
        flexible_default,
        trim,
    })
}
