use crate::utils::*;
use magnus::{block::Yield, value::ReprValue, Error, Ruby, Value};
use std::{collections::VecDeque, io::Read};

/// Parses CSV data from a file and yields each row as a hash to the block.
pub fn parse_csv(
    ruby: &Ruby,
    rb_self: Value,
    args: &[Value],
) -> Result<Yield<impl Iterator<Item = std::collections::HashMap<String, String>>>, Error> {
    if !ruby.block_given() {
        return Ok(Yield::Enumerator(rb_self.enumeratorize("for_each", args)));
    }

    let (to_read, has_headers, delimiter) = parse_csv_args(args)?;
    let (rdr, headers) = setup_csv_parser(ruby, to_read, has_headers, delimiter)?;

    let iter = BufferedRecordsAsHash {
        reader: rdr,
        buffer: VecDeque::with_capacity(1000),
        record: csv::StringRecord::new(),
        headers: headers,
    };

    Ok(Yield::Iter(iter))
}

struct BufferedRecordsAsHash {
    reader: csv::Reader<Box<dyn Read>>,
    buffer: VecDeque<std::collections::HashMap<String, String>>,
    record: csv::StringRecord,
    headers: Vec<String>,
}

impl Iterator for BufferedRecordsAsHash {
    type Item = std::collections::HashMap<String, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            // Refill buffer with up to 1000 records
            while self.buffer.len() < 1000 {
                if !self.reader.read_record(&mut self.record).unwrap() {
                    break;
                }
                let mut map = std::collections::HashMap::new();
                for (i, field) in self.record.iter().enumerate() {
                    let header = if i < self.headers.len() {
                        self.headers[i].to_string()
                    } else {
                        format!("c{}", i)
                    };
                    map.insert(header, field.to_string());
                }
                self.buffer.push_back(map);
            }

            if self.buffer.is_empty() {
                return None;
            }
        }

        self.buffer.pop_front()
    }
}
