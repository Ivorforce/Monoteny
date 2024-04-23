use std::rc::Rc;
use itertools::Itertools;
use uuid::Uuid;
use crate::error::{RResult, RuntimeError};
use crate::interpreter::compiler::compile_deep;
use crate::interpreter::Runtime;
use crate::interpreter::vm::VM;
use crate::program::global::FunctionLogic;
use crate::program::module::Module;
use crate::refactor::Refactor;
use crate::refactor::simplify::Simplify;
use crate::transpiler;
use crate::transpiler::{TranspiledArtifact, Transpiler};

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

    // TODO Should gather all used functions and compile them
    let compiled = compile_deep(runtime, entry_function)?;

    let mut vm = VM::new(&compiled);
    unsafe {
        vm.run()?;
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
    let compiled = compile_deep(runtime, entry_function)?;

    let mut vm = VM::new(&compiled);
    unsafe {
        vm.run()?;
    }

    let exported_artifacts = gather_functions_logic(runtime, &vm.transpile_functions);

    Ok(Box::new(Transpiler {
        // TODO This should be one of the exported artifacts
        main_function: module.main_functions.iter().at_most_one()
            .map_err(|_| RuntimeError::new(format!("Too many main! functions declared: {:?}", module.main_functions)))?
            .cloned(),
        exported_artifacts,
    }))
}

pub fn gather_functions_logic(runtime: &Runtime, transpile_functions: &Vec<Uuid>) -> Vec<TranspiledArtifact> {
    transpile_functions.iter().map(|uuid| {
        let function_head = &runtime.source.fn_heads[uuid];
        match &runtime.source.fn_logic[function_head] {
            FunctionLogic::Implementation(implementation) => {
                // TODO Why copy the implementation now?
                TranspiledArtifact::Function(implementation.clone())
            }
            FunctionLogic::Descriptor(_) => {
                panic!("Cannot transpile a function for which whe don't know an implementation!")
            }
        }
    }).collect()
}
