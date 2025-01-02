use magnus::{IntoValue, Ruby, Value};
use std::{borrow::Cow, collections::HashMap, hash::BuildHasher};

#[derive(Debug)]
pub enum CsvRecord<'a, S: BuildHasher + Default> {
    Vec(Vec<Option<CowValue<'a>>>),
    Map(HashMap<&'static str, Option<CowValue<'a>>, S>),
}

impl<'a, S: BuildHasher + Default> IntoValue for CsvRecord<'a, S> {
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
                map.into_iter()
                    .try_for_each(|(k, v)| hash.aset(k, v))
                    .unwrap();
                hash.into_value_with(handle)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CowValue<'a>(pub Cow<'a, str>);

impl<'a> IntoValue for CowValue<'a> {
    fn into_value_with(self, handle: &Ruby) -> Value {
        self.0.into_value_with(handle)
    }
}
