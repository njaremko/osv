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

    #[inline(always)]
    fn parse(
        headers: &[&'static str],
        record: &csv::StringRecord,
        null_string: Option<&str>,
        flexible_default: Option<&str>,
    ) -> Self::Output {
        let capacity = headers.len();
        let mut map = HashMap::with_capacity(capacity);
        let default = flexible_default.map(String::from);

        while let Some(&header) = headers.iter().next() {
            let value = match record.iter().next() {
                Some(field) => {
                    if Some(field) == null_string {
                        None
                    } else if field.is_empty() {
                        Some(String::new())
                    } else {
                        Some(field.into())
                    }
                }
                None => default.clone(),
            };
            map.insert(header, value);
        }
        map
    }
}

impl RecordParser for Vec<Option<String>> {
    type Output = Self;

    #[inline(always)]
    fn parse(
        headers: &[&'static str],
        record: &csv::StringRecord,
        null_string: Option<&str>,
        flexible_default: Option<&str>,
    ) -> Self::Output {
        let target_len = headers.len();
        let mut vec = Vec::with_capacity(target_len);

        for field in record.iter() {
            let value = if Some(field) == null_string {
                None
            } else if field.is_empty() {
                Some(String::new())
            } else {
                Some(field.into())
            };
            vec.push(value);
        }

        if vec.len() < target_len {
            if let Some(default) = flexible_default {
                vec.resize_with(target_len, || Some(default.to_string()));
            }
        }

        vec
    }
}
