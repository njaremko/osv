use std::collections::HashMap;

pub trait RecordParser {
    type Output;
    fn parse(
        headers: &[&'static str],
        record: &csv::StringRecord,
        null_string: &str,
    ) -> Self::Output;
}

impl RecordParser for HashMap<&'static str, Option<String>> {
    type Output = Self;
    fn parse(
        headers: &[&'static str],
        record: &csv::StringRecord,
        null_string: &str,
    ) -> Self::Output {
        let mut map = HashMap::with_capacity(headers.len());
        for (header, field) in headers.iter().zip(record.iter()) {
            map.insert(
                *header,
                if field == null_string {
                    None
                } else {
                    Some(field.to_string())
                },
            );
        }
        map
    }
}

impl RecordParser for Vec<Option<String>> {
    type Output = Self;
    fn parse(
        _headers: &[&'static str],
        record: &csv::StringRecord,
        null_string: &str,
    ) -> Self::Output {
        let mut vec = Vec::with_capacity(record.len());
        for field in record.iter() {
            vec.push(if field == null_string {
                None
            } else {
                Some(field.to_string())
            });
        }
        vec
    }
}
