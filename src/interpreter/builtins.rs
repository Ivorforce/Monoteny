use std::path::PathBuf;

use crate::error::RResult;
use crate::interpreter::Runtime;
use crate::program::module::module_name;

pub fn load(runtime: &mut Runtime) -> RResult<()> {
    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Monoteny files --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    runtime.repository.add("core", PathBuf::from("monoteny"));
    runtime.get_or_load_module(&module_name("core"))?;

    Ok(())
}
