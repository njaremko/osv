use magnus::{IntoValue, Ruby, Value};
use std::collections::HashMap;

#[derive(Debug)]
pub enum CsvRecord {
    Vec(Vec<Option<String>>),
    Map(HashMap<String, Option<String>>),
}

impl IntoValue for CsvRecord {
    fn into_value_with(self, handle: &Ruby) -> Value {
        match self {
            CsvRecord::Vec(vec) => vec.into_value_with(handle),
            CsvRecord::Map(map) => map.into_value_with(handle),
        }
    }
}
