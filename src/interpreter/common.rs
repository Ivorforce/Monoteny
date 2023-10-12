use std::path::PathBuf;
use crate::error::RuntimeError;
use crate::interpreter::Runtime;

pub fn load(runtime: &mut Runtime) -> Result<(), Vec<RuntimeError>> {
    for name in [
        "precedence", "patterns", "math"
    ] {
        let module = runtime.load_file(&PathBuf::from(format!("monoteny/common/{}.monoteny", name)))?;
        runtime.source.module_order.push(name.to_string());
        runtime.source.module_by_name.insert(name.to_string(), module);
    }

    Ok(())
}
