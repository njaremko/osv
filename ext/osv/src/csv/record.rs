use magnus::{IntoValue, Ruby, Value};
use std::{collections::HashMap, hash::BuildHasher};

#[derive(Debug)]
pub enum CsvRecord<S: BuildHasher + Default> {
    Vec(Vec<Option<String>>),
    Map(HashMap<&'static str, Option<String>, S>),
}

impl<S: BuildHasher + Default> IntoValue for CsvRecord<S> {
    #[inline]
    fn into_value_with(self, handle: &Ruby) -> Value {
        match self {
            CsvRecord::Vec(vec) => vec.into_value_with(handle),
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
