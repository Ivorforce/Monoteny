use std::alloc::{alloc, Layout};
use std::collections::HashMap;
use std::rc::Rc;
use guard::guard;
use crate::error::RResult;
use crate::interpreter::{FunctionInterpreter, Runtime, RuntimeError, Value};
use crate::program::functions::FunctionHead;
use crate::program::module::Module;
use crate::program::traits::RequirementsFulfillment;

pub fn main(module: &Module, runtime: &mut Runtime) -> RResult<()> {
    let entry_function = match &module.main_functions[..] {
        [] => return Err(RuntimeError::new(format!("No @main function declared."))),
        [f] => f,
        functions => return Err(RuntimeError::new(format!("Too many @main functions declared: {:?}", functions))),
    };
    if !entry_function.interface.parameters.is_empty() {
        return Err(RuntimeError::new(format!("@main function has parameters.")));
    }
    if !entry_function.interface.return_type.unit.is_void() {
        return Err(RuntimeError::new(format!("@main function has a return value.")));
    }

    guard!(let Some(implementation) = module.fn_implementations.get(entry_function) else {
        return Err(RuntimeError::new(format!("Cannot run @main function because it does not have a body.")));
    });

    let mut interpreter = FunctionInterpreter {
        runtime,
        implementation: Rc::new(*implementation.clone()),
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
pub fn transpile(module: &Module, runtime: &mut Runtime, callback: &dyn Fn(Rc<FunctionHead>, &Runtime)) -> RResult<()> {
    let entry_function = match &module.transpile_functions[..] {
        [] => return Err(RuntimeError::new(format!("No @transpile function declared."))),
        [f] => f,
        functions => return Err(RuntimeError::new(format!("Too many @main functions declared: {:?}", functions))),
    };
    assert!(entry_function.interface.return_type.unit.is_void(), "@transpile function has a return value.");

    guard!(let Some(implementation) = module.fn_implementations.get(entry_function) else {
        return Err(RuntimeError::new(format!("Cannot run @transpile function because it does not have a body.")));
    });

    let mut assignments = HashMap::new();

    let transpiler_obj = &implementation.parameter_locals[0];

    // Set the transpiler object.
    unsafe {
        let transpiler_layout = Layout::new::<&dyn Fn(Rc<FunctionHead>, &Runtime)>();
        let ptr = alloc(transpiler_layout);
        *(ptr as *mut &dyn Fn(Rc<FunctionHead>, &Runtime)) = callback;
        assignments.insert(
            transpiler_obj.id,
            Value { data: ptr, layout: transpiler_layout }
        );
    }

    let mut interpreter = FunctionInterpreter {
        runtime,
        implementation: Rc::new(*implementation.clone()),
        // TODO Technically we should bind Transpiler here. It should be a subtype of Transpiler
        //  depending on the target language.
        requirements_fulfillment: RequirementsFulfillment::empty(),
        locals: assignments,
    };
    unsafe {
        interpreter.run();
    }

    Ok(())
}


