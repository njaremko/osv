use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::BuildHasher;

use super::header_cache::StringCacheKey;
use super::CowValue;

pub trait RecordParser<'a> {
    type Output: 'a;

    fn parse(
        headers: &[StringCacheKey],
        record: &csv::StringRecord,
        null_string: Option<&'a str>,
        flexible_default: Option<Cow<'a, str>>,
    ) -> Self::Output;
}

impl<'a, S: BuildHasher + Default + 'a> RecordParser<'a>
    for HashMap<StringCacheKey, Option<CowValue<'a>>, S>
{
    type Output = Self;

    #[inline]
    fn parse(
        headers: &[StringCacheKey],
        record: &csv::StringRecord,
        null_string: Option<&str>,
        flexible_default: Option<Cow<'a, str>>,
    ) -> Self::Output {
        let mut map = HashMap::with_capacity_and_hasher(headers.len(), S::default());

        let shared_empty = Cow::Borrowed("");
        let shared_default = flexible_default.map(CowValue);
        headers.iter().enumerate().for_each(|(i, ref header)| {
            let value = record.get(i).map_or_else(
                || shared_default.clone(),
                |field| {
                    if null_string == Some(field) {
                        None
                    } else if field.is_empty() {
                        Some(CowValue(shared_empty.clone()))
                    } else {
                        Some(CowValue(Cow::Owned(field.to_string())))
                    }
                },
            );
            map.insert((*header).clone(), value);
        });
        map
    }
}

impl<'a> RecordParser<'a> for Vec<Option<CowValue<'a>>> {
    type Output = Self;

    #[inline]
    fn parse(
        headers: &[StringCacheKey],
        record: &csv::StringRecord,
        null_string: Option<&str>,
        flexible_default: Option<Cow<'a, str>>,
    ) -> Self::Output {
        let target_len = headers.len();
        let mut vec = Vec::with_capacity(target_len);

        let shared_empty = Cow::Borrowed("");
        let shared_default = flexible_default.map(CowValue);

        for field in record.iter() {
            let value = if Some(field) == null_string {
                None
            } else if field.is_empty() {
                Some(CowValue(shared_empty.clone()))
            } else {
                Some(CowValue(Cow::Owned(field.to_string())))
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
