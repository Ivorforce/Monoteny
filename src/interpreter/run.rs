use std::rc::Rc;

use uuid::Uuid;

use crate::error::{RResult, RuntimeError};
use crate::interpreter::compile::function_compiler::compile_deep;
use crate::interpreter::data::Value;
use crate::interpreter::runtime::Runtime;
use crate::interpreter::vm::VM;
use crate::program::functions::{FunctionHead, FunctionLogic};
use crate::program::module::Module;
use crate::transpiler::{TranspiledArtifact, Transpiler};

pub fn main(module: &Module, runtime: &mut Runtime) -> RResult<()> {
    let entry_function = get_main_function(&module)?
        .ok_or(RuntimeError::error("No main! function declared.").to_array())?;

    // TODO Should gather all used functions and compile them
    let compiled = compile_deep(runtime, entry_function)?;

    let mut out = std::io::stdout();
    let mut vm = VM::new(&mut out);
    unsafe {
        vm.run(compiled, runtime, vec![])?;
    }

    Ok(())
}

pub fn get_main_function(module: &Module) -> RResult<Option<&Rc<FunctionHead>>> {
    let entry_function = match &module.main_functions[..] {
        [] => return Ok(None),
        [f] => f,
        functions => return Err(RuntimeError::error(format!("Too many !main functions declared: {:?}", functions).as_str()).to_array()),
    };
    if !entry_function.interface.parameters.is_empty() {
        return Err(RuntimeError::error("main! function has parameters.").to_array());
    }
    if !entry_function.interface.return_type.unit.is_void() {
        return Err(RuntimeError::error("main! function has a return value.").to_array());
    }
    Ok(Some(entry_function))
}

// The function is written like this
pub fn transpile(module: &Module, runtime: &mut Runtime) -> RResult<Box<Transpiler>> {
    let entry_function = get_transpile_function(module)?;
    assert!(entry_function.interface.return_type.unit.is_void(), "transpile! function has a return value.");

    // Set the transpiler object.
    let compiled = compile_deep(runtime, entry_function)?;

    let mut out = std::io::stdout();
    let mut vm = VM::new(&mut out);
    unsafe {
        vm.run(compiled, runtime, vec![Value { u8: 0 }])?;
    }

    let exported_artifacts = gather_functions_logic(runtime, &vm.transpile_functions);

    Ok(Box::new(Transpiler {
        // TODO This should be one of the exported artifacts
        main_function: get_main_function(module)?.map(Rc::clone),
        exported_artifacts,
    }))
}

fn get_transpile_function(module: &Module) -> RResult<&Rc<FunctionHead>> {
    match &module.transpile_functions[..] {
        [] => Err(RuntimeError::error("No transpile! function declared.").to_array()),
        [f] => Ok(f),
        functions => Err(RuntimeError::error(format!("Too many transpile! functions declared: {:?}", functions).as_str()).to_array()),
    }
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
