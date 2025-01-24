use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::BuildHasher;

use super::header_cache::StringCacheKey;
use super::CowStr;

pub trait RecordParser<'a> {
    type Output;

    fn parse(
        headers: &[StringCacheKey],
        record: &csv::StringRecord,
        null_string: Option<Cow<'a, str>>,
        flexible_default: Option<Cow<'a, str>>,
        ignore_null_bytes: bool,
    ) -> Self::Output;
}

impl<'a, S: BuildHasher + Default> RecordParser<'a>
    for HashMap<StringCacheKey, Option<CowStr<'a>>, S>
{
    type Output = Self;

    #[inline]
    fn parse(
        headers: &[StringCacheKey],
        record: &csv::StringRecord,
        null_string: Option<Cow<'a, str>>,
        flexible_default: Option<Cow<'a, str>>,
        ignore_null_bytes: bool,
    ) -> Self::Output {
        let mut map = HashMap::with_capacity_and_hasher(headers.len(), S::default());

        let shared_empty = Cow::Borrowed("");
        let shared_default = flexible_default.map(CowStr);
        headers.iter().enumerate().for_each(|(i, header)| {
            let value = record.get(i).map_or_else(
                || shared_default.clone(),
                |field| {
                    if null_string.as_deref() == Some(field) {
                        None
                    } else if field.is_empty() {
                        Some(CowStr(shared_empty.clone()))
                    } else if ignore_null_bytes  {
                        Some(CowStr(Cow::Owned(field.replace("\0", "").to_string())))
                    }
                    else {
                        Some(CowStr(Cow::Owned(field.to_string())))
                    }
                },
            );
            map.insert(*header, value);
        });
        map
    }
}

impl<'a> RecordParser<'a> for Vec<Option<CowStr<'a>>> {
    type Output = Self;

    #[inline]
    fn parse(
        headers: &[StringCacheKey],
        record: &csv::StringRecord,
        null_string: Option<Cow<'a, str>>,
        flexible_default: Option<Cow<'a, str>>,
        ignore_null_bytes: bool,
    ) -> Self::Output {
        let target_len = headers.len();
        let mut vec = Vec::with_capacity(target_len);

        let shared_empty = Cow::Borrowed("");
        let shared_default = flexible_default.map(CowStr);

        for field in record.iter() {
            let value = if Some(field) == null_string.as_deref() {
                None
            } else if field.is_empty() {
                Some(CowStr(shared_empty.clone()))
            } else if ignore_null_bytes  {
                Some(CowStr(Cow::Owned(field.replace("\0", "").to_string())))
            }
            else {
                Some(CowStr(Cow::Owned(field.to_string())))
            };
            vec.push(value);
        }

        if vec.len() < target_len {
            if let Some(default) = shared_default {
                vec.resize_with(target_len, || Some(default.clone()));
            }
        }
        vec
    }
}
