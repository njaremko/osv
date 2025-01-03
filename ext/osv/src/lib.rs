mod allocator;
mod csv;
mod reader;
mod utils;

use crate::reader::*;

use magnus::{Error, Ruby};

/// Initializes the Ruby extension and defines methods.
#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("OSV")?;
    module.define_module_function("for_each", magnus::method!(parse_csv, -1))?;
    Ok(())
}
