use std::collections::HashMap;

pub trait RecordParser {
    type Output;

    fn parse(headers: &[String], record: &csv::StringRecord, null_string: &str) -> Self::Output;
}

impl RecordParser for HashMap<String, Option<String>> {
    type Output = Self;

    fn parse(headers: &[String], record: &csv::StringRecord, null_string: &str) -> Self::Output {
        headers
            .iter()
            .zip(record.iter())
            .map(|(header, field)| {
                let value = if field == null_string {
                    None
                } else {
                    Some(field.to_string())
                };
                (header.clone(), value)
            })
            .collect()
    }
}

impl RecordParser for Vec<Option<String>> {
    type Output = Self;

    fn parse(_headers: &[String], record: &csv::StringRecord, null_string: &str) -> Self::Output {
        record
            .iter()
            .map(|field| {
                if field == null_string {
                    None
                } else {
                    Some(field.to_string())
                }
            })
            .collect()
    }
}
