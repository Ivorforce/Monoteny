use std::alloc::{alloc, Layout};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::rc::Rc;

use itertools::Itertools;

use crate::error::RResult;
use crate::interpreter::{FunctionInterpreter, Runtime, RuntimeError};
use crate::interpreter::allocation::Value;
use crate::program::functions::FunctionHead;
use crate::program::global::FunctionLogic;
use crate::program::module::Module;
use crate::program::traits::RequirementsFulfillment;
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

    let FunctionLogic::Implementation(implementation) = &runtime.source.fn_logic[entry_function] else {
        return Err(RuntimeError::new(format!("Cannot run main! function because it is not implemented.")));
    };

    let mut interpreter = FunctionInterpreter {
        implementation: Rc::new(*implementation.clone()),
        runtime,
        // No parameters and return type = nothing to bind!
        requirements_fulfillment: RequirementsFulfillment::empty(),
        locals: HashMap::new(),
    };
    unsafe {
        interpreter.run();
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

    let FunctionLogic::Implementation(implementation) = &runtime.source.fn_logic[entry_function] else {
        return Err(RuntimeError::new(format!("Cannot run transpile! function because it does not have a body.")));
    };

    let mut assignments = HashMap::new();
    let mut transpiler = Rc::new(RefCell::new(Box::new(Transpiler {
        main_function: module.main_functions.iter().at_most_one()
            .map_err(|_| RuntimeError::new(format!("Too many main! functions declared: {:?}", module.main_functions)))?
            .cloned(),
        exported_artifacts: vec![],
    })));

    let transpiler_obj_ref = &implementation.parameter_locals[0];

    let callback = |function_head, runtime: &Runtime| {
        let mut transpiler = transpiler.borrow_mut();
        let transpiler = transpiler.deref_mut();

        match &runtime.source.fn_logic[&function_head] {
            FunctionLogic::Implementation(implementation) => {
                transpiler.exported_artifacts.push(TranspiledArtifact::Function(implementation.clone()));
            }
            FunctionLogic::Descriptor(_) => {
                panic!("Cannot transpile a function for which whe don't know an implementation!")
            }
        }
    };

    // Set the transpiler object.
    unsafe {
        let transpiler_layout = Layout::new::<&dyn Fn(Rc<FunctionHead>, &Runtime)>();
        let ptr = alloc(transpiler_layout);
        *(ptr as *mut &dyn Fn(Rc<FunctionHead>, &Runtime)) = &callback;
        assignments.insert(
            transpiler_obj_ref.id,
            Value { data: ptr, layout: transpiler_layout }
        );
    }

    let mut interpreter = FunctionInterpreter {
        implementation: Rc::new(*implementation.clone()),
        runtime,
        // TODO Technically we should bind Transpiler here. It should be a subtype of Transpiler
        //  depending on the target language.
        requirements_fulfillment: RequirementsFulfillment::empty(),
        locals: assignments,
    };
    unsafe {
        interpreter.run();
    }

    Ok(Rc::try_unwrap(transpiler).map_err(|_| ()).expect("Internal Error on try_unwrap(exported_artifacts)").into_inner())
}


