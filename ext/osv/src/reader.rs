use crate::csv::{CowStr, CsvRecord, RecordReaderBuilder, StringCacheKey};
use crate::utils::*;
use ahash::RandomState;
use csv::Trim;
use magnus::value::ReprValue;
use magnus::{Error, IntoValue, KwArgs, RHash, Ruby, Symbol, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// Valid result types for CSV parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResultType {
    Hash,
    Array,
}

impl ResultType {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "hash" => Some(Self::Hash),
            "array" => Some(Self::Array),
            _ => None,
        }
    }
}

/// Arguments for creating an enumerator
#[derive(Debug)]
struct EnumeratorArgs {
    rb_self: Value,
    to_read: Value,
    has_headers: bool,
    delimiter: u8,
    quote_char: u8,
    null_string: Option<String>,
    result_type: String,
    flexible: bool,
    trim: Option<String>,
    ignore_null_bytes: bool,
    lossy: bool,
}

/// Parses a CSV file with the given configuration.
///
/// # Safety
/// This function uses unsafe code to get the Ruby runtime and leak memory for static references.
/// This is necessary for Ruby integration but should be used with caution.
pub fn parse_csv(rb_self: Value, args: &[Value]) -> Result<Value, Error> {
    //  SAFETY: We're in a Ruby callback, so Ruby runtime is guaranteed to be initialized
    let ruby = unsafe { Ruby::get_unchecked() };

    let ReadCsvArgs {
        to_read,
        has_headers,
        delimiter,
        quote_char,
        null_string,
        result_type,
        flexible,
        trim,
        ignore_null_bytes,
        lossy,
    } = parse_read_csv_args(&ruby, args)?;

    if !ruby.block_given() {
        return create_enumerator(EnumeratorArgs {
            rb_self,
            to_read,
            has_headers,
            delimiter,
            quote_char,
            null_string,
            result_type,
            flexible,
            trim: match trim {
                Trim::All => Some("all".to_string()),
                Trim::Headers => Some("headers".to_string()),
                Trim::Fields => Some("fields".to_string()),
                _ => None,
            },
            ignore_null_bytes,
            lossy,
        })
        .map(|yield_enum| yield_enum.into_value_with(&ruby));
    }

    let result_type = ResultType::from_str(&result_type).ok_or_else(|| {
        Error::new(
            ruby.exception_runtime_error(),
            "Invalid result type, expected 'hash' or 'array'",
        )
    })?;

    match result_type {
        ResultType::Hash => {
            let builder = RecordReaderBuilder::<
                HashMap<Arc<StringCacheKey>, Option<CowStr<'_>>, RandomState>,
            >::new(ruby, to_read)
            .has_headers(has_headers)
            .flexible(flexible)
            .trim(trim)
            .delimiter(delimiter)
            .quote_char(quote_char)
            .null_string(null_string)
            .ignore_null_bytes(ignore_null_bytes)
            .lossy(lossy)
            .build()?;

            let ruby = unsafe { Ruby::get_unchecked() };
            for result in builder {
                let record = result?;
                let _: Value = ruby.yield_value(CsvRecord::Map(record))?;
            }
        }
        ResultType::Array => {
            let builder = RecordReaderBuilder::<Vec<Option<CowStr<'_>>>>::new(ruby, to_read)
                .has_headers(has_headers)
                .flexible(flexible)
                .trim(trim)
                .delimiter(delimiter)
                .quote_char(quote_char)
                .null_string(null_string)
                .ignore_null_bytes(ignore_null_bytes)
                .lossy(lossy)
                .build()?;

            let ruby = unsafe { Ruby::get_unchecked() };
            for result in builder {
                let record = result?;
                let _: Value = ruby.yield_value(CsvRecord::<ahash::RandomState>::Vec(record))?;
            }
        }
    }

    let ruby = unsafe { Ruby::get_unchecked() };
    Ok(ruby.qnil().into_value_with(&ruby))
}

/// Creates an enumerator for lazy CSV parsing
fn create_enumerator(args: EnumeratorArgs) -> Result<magnus::Enumerator, Error> {
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
    kwargs.aset(Symbol::new("result_type"), Symbol::new(args.result_type))?;
    kwargs.aset(Symbol::new("flexible"), args.flexible)?;
    kwargs.aset(Symbol::new("trim"), args.trim.map(Symbol::new))?;
    kwargs.aset(Symbol::new("ignore_null_bytes"), args.ignore_null_bytes)?;
    kwargs.aset(Symbol::new("lossy"), args.lossy)?;
    Ok(args
        .rb_self
        .enumeratorize("for_each", (args.to_read, KwArgs(kwargs))))
}
