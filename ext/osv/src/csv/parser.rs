use std::collections::HashMap;

pub trait RecordParser {
    type Output;

    fn parse(
        headers: &[&'static str],
        record: &csv::StringRecord,
        null_string: Option<&str>,
    ) -> Self::Output;
}

impl RecordParser for HashMap<&'static str, Option<String>> {
    type Output = Self;

    #[inline]
    fn parse(
        headers: &[&'static str],
        record: &csv::StringRecord,
        null_string: Option<&str>,
    ) -> Self::Output {
        let mut map = HashMap::with_capacity(headers.len());
        headers
            .iter()
            .zip(record.iter())
            .for_each(|(header, field)| {
                map.insert(
                    *header,
                    if null_string == Some(field) {
                        None
                    } else {
                        // Avoid allocating for empty strings
                        if field.is_empty() {
                            Some(String::new())
                        } else {
                            Some(field.to_string())
                        }
                    },
                );
            });
        map
    }
}

impl RecordParser for Vec<Option<String>> {
    type Output = Self;

    #[inline]
    fn parse(
        _headers: &[&'static str],
        record: &csv::StringRecord,
        null_string: Option<&str>,
    ) -> Self::Output {
        let mut vec = Vec::with_capacity(record.len());
        vec.extend(record.iter().map(|field| {
            if null_string == Some(field) {
                None
            } else {
                // Avoid allocating for empty strings
                if field.is_empty() {
                    Some(String::new())
                } else {
                    Some(field.to_string())
                }
            }
        }));
        vec
    }
}
