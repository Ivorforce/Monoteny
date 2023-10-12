use std::path::PathBuf;
use crate::error::RuntimeError;
use crate::interpreter::Runtime;

pub fn load(runtime: &mut Runtime) -> Result<(), Vec<RuntimeError>> {
    for name in [
        "precedence", "patterns", "debug", "math"
    ] {
        let module_name = format!("common.{}", name);
        let module = runtime.load_file(&PathBuf::from(format!("monoteny/common/{}.monoteny", name)))?;
        runtime.source.module_order.push(module_name.clone());
        runtime.source.module_by_name.insert(module_name, module);
    }

    Ok(())
}
