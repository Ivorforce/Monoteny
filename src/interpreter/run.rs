use std::alloc::{alloc, Layout};
use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;
use crate::interpreter::{FunctionInterpreter, InterpreterGlobals, Value};
use crate::program::{find_annotated, Program};
use crate::program::traits::RequirementsFulfillment;

pub fn main(program: &Program, globals: &mut InterpreterGlobals) {
    let entry_function = find_annotated(program.module.function_implementations.values(), "main").expect("No main function!");
    assert!(entry_function.head.interface.parameters.is_empty(), "@main function has parameters.");
    assert!(entry_function.head.interface.return_type.unit.is_void(), "@main function has a return value.");

    let mut interpreter = FunctionInterpreter {
        globals,
        implementation: Rc::clone(entry_function),
        // No parameters and return type = nothing to bind!
        requirements_fulfillment: RequirementsFulfillment::empty(),
        locals: HashMap::new(),
    };
    unsafe {
        interpreter.run();
    }
}

pub fn transpile(program: &Program, globals: &mut InterpreterGlobals, callback: &dyn Fn(Uuid)) {
    let entry_function = find_annotated(program.module.function_implementations.values(), "transpile").expect("No main function!");
    assert!(entry_function.head.interface.return_type.unit.is_void(), "@transpile function has a return value.");

    let mut assignments = HashMap::new();

    let transpiler_obj = &entry_function.parameter_variables[0];

    // Set the transpiler object.
    unsafe {
        let transpiler_layout = Layout::new::<&dyn Fn(Uuid)>();
        let ptr = alloc(transpiler_layout);
        *(ptr as *mut &dyn Fn(Uuid)) = callback;
        assignments.insert(
            transpiler_obj.id,
            Value { data: ptr, layout: transpiler_layout }
        );
    }

    let mut interpreter = FunctionInterpreter {
        globals,
        implementation: Rc::clone(entry_function),
        // TODO Technically we should bind Transpiler here, probably to a Transpiler subtype that cannot be instantiated.
        requirements_fulfillment: RequirementsFulfillment::empty(),
        locals: assignments,
    };
    unsafe {
        interpreter.run();
    }
}


