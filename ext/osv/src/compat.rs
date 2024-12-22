use crate::utils::*;
use magnus::{block::Yield, value::ReprValue, Error, Ruby, Value};
use std::{collections::VecDeque, io::Read};

pub fn parse_compat(
    ruby: &Ruby,
    rb_self: Value,
    args: &[Value],
) -> Result<Yield<impl Iterator<Item = Vec<String>>>, Error> {
    if !ruby.block_given() {
        return Ok(Yield::Enumerator(
            rb_self.enumeratorize("for_each_compat", args),
        ));
    }

    let (to_read, has_headers, delimiter) = parse_csv_args(args)?;
    let (rdr, _) = setup_csv_parser(ruby, to_read, has_headers, delimiter)?;

    let iter = BufferedRecords {
        reader: rdr,
        buffer: VecDeque::with_capacity(1000),
        record: csv::StringRecord::new(),
    };

    Ok(Yield::Iter(iter))
}

struct BufferedRecords {
    reader: csv::Reader<Box<dyn Read>>,
    buffer: VecDeque<Vec<String>>,
    record: csv::StringRecord,
}

impl Iterator for BufferedRecords {
    type Item = Vec<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            // Refill buffer with up to 1000 records
            while self.buffer.len() < 1000 {
                if !self.reader.read_record(&mut self.record).unwrap() {
                    break;
                }
                let row = self.record.iter().map(|field| field.to_string()).collect();
                self.buffer.push_back(row);
            }

            if self.buffer.is_empty() {
                return None;
            }
        }

        self.buffer.pop_front()
    }
}
