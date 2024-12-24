use magnus::{IntoValue, Ruby, Value};
use std::collections::HashMap;

#[derive(Debug)]
pub enum CsvRecord {
    Vec(Vec<Option<String>>),
    Map(HashMap<&'static str, Option<String>>),
}

impl IntoValue for CsvRecord {
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
