use crate::csv::{CowValue, CsvRecord, RecordReaderBuilder, StringCacheKey};
use crate::utils::*;
use ahash::RandomState;
use csv::Trim;
use magnus::value::ReprValue;
use magnus::{block::Yield, Error, KwArgs, RHash, Ruby, Symbol, Value};
use std::collections::HashMap;

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
    flexible_default: Option<String>,
    trim: Option<String>,
}

/// Parses a CSV file with the given configuration.
///
/// # Safety
/// This function uses unsafe code to get the Ruby runtime and leak memory for static references.
/// This is necessary for Ruby integration but should be used with caution.
pub fn parse_csv(
    rb_self: Value,
    args: &[Value],
) -> Result<Yield<Box<dyn Iterator<Item = CsvRecord<'static, RandomState>>>>, Error> {
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
        flexible_default,
        trim,
    } = parse_read_csv_args(&ruby, args)?;

    if !ruby.block_given() {
        return create_enumerator(EnumeratorArgs {
            rb_self,
            to_read,
            has_headers,
            delimiter,
            quote_char,
            null_string,
            result_type: result_type,
            flexible,
            flexible_default: flexible_default,
            trim: match trim {
                Trim::All => Some("all".to_string()),
                Trim::Headers => Some("headers".to_string()),
                Trim::Fields => Some("fields".to_string()),
                _ => None,
            },
        });
    }

    let result_type = ResultType::from_str(&result_type).ok_or_else(|| {
        Error::new(
            ruby.exception_runtime_error(),
            "Invalid result type, expected 'hash' or 'array'",
        )
    })?;

    let iter: Box<dyn Iterator<Item = CsvRecord<RandomState>>> = match result_type {
        ResultType::Hash => {
            let builder = RecordReaderBuilder::<
                HashMap<StringCacheKey, Option<CowValue<'static>>, RandomState>,
            >::new(ruby, to_read)
            .has_headers(has_headers)
            .flexible(flexible)
            .flexible_default(flexible_default)
            .trim(trim)
            .delimiter(delimiter)
            .quote_char(quote_char)
            .null_string(null_string);

            Box::new(builder.build()?.map(CsvRecord::Map))
        }
        ResultType::Array => {
            let builder = RecordReaderBuilder::<Vec<Option<CowValue<'static>>>>::new(ruby, to_read)
                .has_headers(has_headers)
                .flexible(flexible)
                .flexible_default(flexible_default)
                .trim(trim)
                .delimiter(delimiter)
                .quote_char(quote_char)
                .null_string(null_string)
                .build()?;

            Box::new(builder.map(CsvRecord::Vec))
        }
    };

    Ok(Yield::Iter(iter))
}

/// Creates an enumerator for lazy CSV parsing
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
    kwargs.aset(Symbol::new("result_type"), Symbol::new(args.result_type))?;
    kwargs.aset(Symbol::new("flexible"), args.flexible)?;
    kwargs.aset(Symbol::new("flexible_default"), args.flexible_default)?;
    kwargs.aset(Symbol::new("trim"), args.trim.map(Symbol::new))?;

    let enumerator = args
        .rb_self
        .enumeratorize("for_each", (args.to_read, KwArgs(kwargs)));
    Ok(Yield::Enumerator(enumerator))
}
