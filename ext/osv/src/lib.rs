mod compat;
mod hash;
mod io_reader;
mod utils;

use crate::compat::*;
use crate::hash::*;

use magnus::{Error, Ruby};

/// Initializes the Ruby extension and defines methods.
#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("OSV")?;
    module.define_module_function("for_each", magnus::method!(parse_csv, -1))?;
    module.define_module_function("for_each_compat", magnus::method!(parse_compat, -1))?;
    Ok(())
}
