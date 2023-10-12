use std::path::PathBuf;
use crate::error::{RResult, RuntimeError};
use crate::interpreter::Runtime;
use crate::program::module::module_name;

pub fn load(runtime: &mut Runtime) -> RResult<()> {
    let module = runtime.load_file(&PathBuf::from("monoteny/common.monoteny"), module_name("common"))?;
    runtime.source.module_by_name.insert(module.name.clone(), module);

    Ok(())
}
