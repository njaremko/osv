use std::collections::HashMap;

pub trait RecordParser {
    type Output;

    fn parse(
        headers: &[&'static str],
        record: &csv::StringRecord,
        null_string: Option<&str>,
        flexible_default: Option<&str>,
    ) -> Self::Output;
}

impl RecordParser for HashMap<&'static str, Option<String>> {
    type Output = Self;

    #[inline]
    fn parse(
        headers: &[&'static str],
        record: &csv::StringRecord,
        null_string: Option<&str>,
        flexible_default: Option<&str>,
    ) -> Self::Output {
        let mut map = HashMap::with_capacity(headers.len());
        headers.iter().enumerate().for_each(|(i, &header)| {
            let value = record.get(i).map_or_else(
                || flexible_default.map(ToString::to_string),
                |field| {
                    if null_string == Some(field) {
                        None
                    } else if field.is_empty() {
                        Some(String::new())
                    } else {
                        Some(field.to_string())
                    }
                },
            );
            map.insert(header, value);
        });
        map
    }
}

impl RecordParser for Vec<Option<String>> {
    type Output = Self;

    #[inline]
    fn parse(
        headers: &[&'static str],
        record: &csv::StringRecord,
        null_string: Option<&str>,
        flexible_default: Option<&str>,
    ) -> Self::Output {
        let target_len = headers.len();
        let mut vec = Vec::with_capacity(target_len);

        vec.extend(record.iter().map(|field| {
            if null_string == Some(field) {
                None
            } else if field.is_empty() {
                Some(String::new())
            } else {
                Some(field.to_string())
            }
        }));

        if let Some(default) = flexible_default {
            let default = default.to_string();
            vec.resize_with(target_len, || Some(default.clone()));
        }
        vec
    }
}
