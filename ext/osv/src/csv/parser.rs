use super::builder::ReaderError;
use super::header_cache::StringCacheKey;
use super::CowStr;
use magnus::Ruby;
use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::BuildHasher;

pub enum CsvRecordType {
    String(csv::StringRecord),
    Byte(csv::ByteRecord),
}

pub trait RecordParser<'a> {
    type Output;

    fn parse(
        handle: &Ruby,
        headers: &[StringCacheKey],
        record: &CsvRecordType,
        null_string: Option<Cow<'a, str>>,
        ignore_null_bytes: bool,
    ) -> Result<Self::Output, ReaderError>;

    fn uses_headers() -> bool;
}

impl<'a, S: BuildHasher + Default> RecordParser<'a>
    for HashMap<&'static str, Option<CowStr<'a>>, S>
{
    type Output = Self;

    #[inline]
    fn uses_headers() -> bool {
        true
    }

    #[inline]
    fn parse(
        handle: &Ruby,
        headers: &[StringCacheKey],
        record: &CsvRecordType,
        null_string: Option<Cow<'a, str>>,
        ignore_null_bytes: bool,
    ) -> Result<Self::Output, ReaderError> {
        let mut map = HashMap::with_capacity_and_hasher(headers.len(), S::default());
        let shared_empty = Cow::Borrowed("");

        for (i, header) in headers.iter().enumerate() {
            let value = match record {
                CsvRecordType::String(s) => s.get(i).and_then(|field| {
                    convert_field_to_cow_str(
                        field,
                        null_string.as_deref(),
                        ignore_null_bytes,
                        &shared_empty,
                    )
                }),
                CsvRecordType::Byte(b) => b.get(i).and_then(|field| {
                    let field = String::from_utf8_lossy(field);
                    convert_field_to_cow_str(
                        &field,
                        null_string.as_deref(),
                        ignore_null_bytes,
                        &shared_empty,
                    )
                }),
            };

            map.insert(header.as_str(handle)?, value);
        }

        Ok(map)
    }
}

impl<'a> RecordParser<'a> for Vec<Option<CowStr<'a>>> {
    type Output = Self;

    #[inline]
    fn uses_headers() -> bool {
        false
    }

    #[inline]
    fn parse(
        _handle: &Ruby,
        headers: &[StringCacheKey],
        record: &CsvRecordType,
        null_string: Option<Cow<'a, str>>,
        ignore_null_bytes: bool,
    ) -> Result<Self::Output, ReaderError> {
        let target_len = headers.len();
        let mut vec = Vec::with_capacity(target_len);
        let shared_empty = Cow::Borrowed("");

        match record {
            CsvRecordType::String(record) => {
                for field in record.iter() {
                    let value = convert_field_to_cow_str(
                        field,
                        null_string.as_deref(),
                        ignore_null_bytes,
                        &shared_empty,
                    );
                    vec.push(value);
                }
            }
            CsvRecordType::Byte(record) => {
                for field in record.iter() {
                    let field = String::from_utf8_lossy(field);
                    let value = convert_field_to_cow_str(
                        &field,
                        null_string.as_deref(),
                        ignore_null_bytes,
                        &shared_empty,
                    );
                    vec.push(value);
                }
            }
        }

        Ok(vec)
    }
}

#[inline]
fn convert_field_to_cow_str<'a>(
    field: &str,
    null_string: Option<&str>,
    ignore_null_bytes: bool,
    shared_empty: &Cow<'a, str>,
) -> Option<CowStr<'a>> {
    if Some(field) == null_string {
        None
    } else if field.is_empty() {
        Some(CowStr(shared_empty.clone()))
    } else if ignore_null_bytes {
        Some(CowStr(Cow::Owned(field.replace("\0", ""))))
    } else {
        Some(CowStr(Cow::Owned(field.to_string())))
    }
}
