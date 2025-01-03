use crate::csv::{CowValue, CsvRecord, RecordReaderBuilder};
use crate::utils::*;
use ahash::RandomState;
use csv::Trim;
use magnus::value::ReprValue;
use magnus::{block::Yield, Error, KwArgs, RHash, Ruby, Symbol, Value};
use std::collections::HashMap;

pub fn parse_csv(
    rb_self: Value,
    args: &[Value],
) -> Result<Yield<Box<dyn Iterator<Item = CsvRecord<'static, RandomState>>>>, Error> {
    let original = unsafe { Ruby::get_unchecked() };
    let ruby: &'static Ruby = Box::leak(Box::new(original));

    let ReadCsvArgs {
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
    } = parse_read_csv_args(ruby, args)?;

    let flexible_default: &'static Option<String> = Box::leak(Box::new(flexible_default));
    let leaked_flexible_default: &'static Option<&str> =
        Box::leak(Box::new(flexible_default.as_deref()));

    if !ruby.block_given() {
        return create_enumerator(EnumeratorArgs {
            rb_self,
            to_read,
            has_headers,
            delimiter,
            quote_char,
            null_string,
            buffer_size,
            result_type,
            flexible,
            flexible_default: leaked_flexible_default.as_deref(),
            trim: match trim {
                Trim::All => Some("all".to_string()),
                Trim::Headers => Some("headers".to_string()),
                Trim::Fields => Some("fields".to_string()),
                _ => None,
            },
        });
    }

    let iter: Box<dyn Iterator<Item = CsvRecord<RandomState>>> = match result_type.as_str() {
        "hash" => {
            let builder = RecordReaderBuilder::<
                HashMap<&'static str, Option<CowValue<'static>>, RandomState>,
            >::new(ruby, to_read)
            .has_headers(has_headers)
            .flexible(flexible)
            .flexible_default(flexible_default.as_deref())
            .trim(trim)
            .delimiter(delimiter)
            .quote_char(quote_char)
            .null_string(null_string)
            .buffer(buffer_size);

            Box::new(builder.build_threaded()?.map(CsvRecord::Map))
        }
        "array" => Box::new(
            RecordReaderBuilder::<Vec<Option<CowValue<'static>>>>::new(ruby, to_read)
                .has_headers(has_headers)
                .flexible(flexible)
                .flexible_default(flexible_default.as_deref())
                .trim(trim)
                .delimiter(delimiter)
                .quote_char(quote_char)
                .null_string(null_string)
                .buffer(buffer_size)
                .build_threaded()?
                .map(CsvRecord::Vec),
        ),
        _ => {
            return Err(Error::new(
                ruby.exception_runtime_error(),
                "Invalid result type",
            ))
        }
    };

    Ok(Yield::Iter(iter))
}

struct EnumeratorArgs {
    rb_self: Value,
    to_read: Value,
    has_headers: bool,
    delimiter: u8,
    quote_char: u8,
    null_string: Option<String>,
    buffer_size: usize,
    result_type: String,
    flexible: bool,
    flexible_default: Option<&'static str>,
    trim: Option<String>,
}

fn create_enumerator(
    args: EnumeratorArgs,
) -> Result<Yield<Box<dyn Iterator<Item = CsvRecord<'static, RandomState>>>>, Error> {
    let kwargs = RHash::new();
    kwargs.aset(Symbol::new("has_headers"), args.has_headers)?;
    kwargs.aset(
        Symbol::new("col_sep"),
        String::from_utf8(vec![args.delimiter]).unwrap(),
    )?;
    kwargs.aset(
        Symbol::new("quote_char"),
        String::from_utf8(vec![args.quote_char]).unwrap(),
    )?;
    kwargs.aset(Symbol::new("nil_string"), args.null_string)?;
    kwargs.aset(Symbol::new("buffer_size"), args.buffer_size)?;
    kwargs.aset(Symbol::new("result_type"), Symbol::new(args.result_type))?;
    kwargs.aset(Symbol::new("flexible"), args.flexible)?;
    kwargs.aset(Symbol::new("flexible_default"), args.flexible_default)?;
    kwargs.aset(Symbol::new("trim"), args.trim.map(Symbol::new))?;
    let enumerator = args
        .rb_self
        .enumeratorize("for_each", (args.to_read, KwArgs(kwargs)));
    Ok(Yield::Enumerator(enumerator))
}
