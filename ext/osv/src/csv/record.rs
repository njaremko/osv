use magnus::{IntoValue, RArray, Ruby, Value};
use std::collections::HashMap;

#[derive(Debug)]
pub enum CsvRecord {
    Vec(Vec<Option<String>>),
    Map(HashMap<&'static str, Option<String>>),
}

impl IntoValue for CsvRecord {
    #[inline(always)]
    fn into_value_with(self, handle: &Ruby) -> Value {
        match self {
            CsvRecord::Vec(vec) => {
                let ary = RArray::with_capacity(vec.len());

                for opt_str in vec {
                    let _ = ary.push(opt_str);
                }

                ary.into_value_with(handle)
            }
            CsvRecord::Map(map) => {
                let hash = handle.hash_new_capa(map.len());

                for (k, v) in map {
                    let _ = hash.aset(k, v);
                }

                hash.into_value_with(handle)
            }
        }
    }
}
