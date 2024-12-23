use magnus::{
    scan_args::{get_kwargs, scan_args},
    Error, Value,
};

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
