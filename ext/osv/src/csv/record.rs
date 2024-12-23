use magnus::{IntoValue, RHash, Ruby, Value};
use std::collections::HashMap;

#[derive(Debug)]
pub enum CsvRecord {
    Vec(Vec<Option<String>>),
    Map(HashMap<&'static str, Option<String>>),
}

impl IntoValue for CsvRecord {
    fn into_value_with(self, handle: &Ruby) -> Value {
        match self {
            CsvRecord::Vec(vec) => vec.into_value_with(handle),
            CsvRecord::Map(map) => {
                let hash = RHash::new();
                for (k, v) in map {
                    hash.aset(k, v).unwrap();
                }
                hash.into_value_with(handle)
            }
        }
    }
}
