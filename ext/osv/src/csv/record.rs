use itertools::Itertools;
use magnus::{value::ReprValue, IntoValue, Ruby, Value};
use std::{borrow::Cow, collections::HashMap, hash::BuildHasher};

use super::StringCacheKey;

#[derive(Debug)]
pub enum CsvRecord<'a, S: BuildHasher + Default> {
    Vec(Vec<Option<CowStr<'a>>>),
    Map(HashMap<StringCacheKey, Option<CowStr<'a>>, S>),
}

impl<S: BuildHasher + Default> IntoValue for CsvRecord<'_, S> {
    #[inline]
    fn into_value_with(self, handle: &Ruby) -> Value {
        match self {
            CsvRecord::Vec(vec) => {
                let ary = handle.ary_new_capa(vec.len());
                vec.into_iter().try_for_each(|v| ary.push(v)).unwrap();
                ary.into_value_with(handle)
            }
            CsvRecord::Map(map) => {
                // Pre-allocate the hash with the known size
                let hash = handle.hash_new_capa(map.len());

                let mut values: [Value; 128] = [handle.qnil().as_value(); 128];
                let mut i = 0;

                for chunk in &map.into_iter().chunks(64) {
                    for (k, v) in chunk {
                        values[i] = handle.into_value(k.as_ref());
                        values[i + 1] = handle.into_value(v);
                        i += 2;
                    }
                    hash.bulk_insert(&values[..i]).unwrap();

                    // Zero out used values
                    values[..i].fill(handle.qnil().as_value());
                    i = 0;
                }

                hash.into_value_with(handle)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CowStr<'a>(pub Cow<'a, str>);

impl IntoValue for CowStr<'_> {
    fn into_value_with(self, handle: &Ruby) -> Value {
        self.0.into_value_with(handle)
    }
}
