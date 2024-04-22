use crate::error::{RResult, RuntimeError};
use crate::interpreter::compiler::compile;
use crate::interpreter::Runtime;
use crate::interpreter::vm::VM;
use crate::program::module::Module;
use crate::transpiler::Transpiler;

pub fn main(module: &Module, runtime: &mut Runtime) -> RResult<()> {
    let entry_function = match &module.main_functions[..] {
        [] => return Err(RuntimeError::new(format!("No main! function declared."))),
        [f] => f,
        functions => return Err(RuntimeError::new(format!("Too many @main functions declared: {:?}", functions))),
    };
    if !entry_function.interface.parameters.is_empty() {
        return Err(RuntimeError::new(format!("main! function has parameters.")));
    }
    if !entry_function.interface.return_type.unit.is_void() {
        return Err(RuntimeError::new(format!("main! function has a return value.")));
    }

    let compiled = compile(runtime, entry_function)?;

    let mut vm = VM {
        chunk: compiled,
        stack: vec![],
    };
    unsafe {
        vm.run();
    }

    Ok(())
}

// The function is written like this
pub fn transpile(module: &Module, runtime: &mut Runtime) -> RResult<Box<Transpiler>> {
    let entry_function = match &module.transpile_functions[..] {
        [] => return Err(RuntimeError::new(format!("No transpile! function declared."))),
        [f] => f,
        functions => return Err(RuntimeError::new(format!("Too many main! functions declared: {:?}", functions))),
    };
    assert!(entry_function.interface.return_type.unit.is_void(), "transpile! function has a return value.");

    // Set the transpiler object.
    let compiled = compile(runtime, entry_function)?;

    let mut vm = VM {
        chunk: compiled,
        stack: vec![],
    };
    unsafe {
        vm.run();
    }

    todo!()
}
