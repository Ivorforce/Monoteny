use std::alloc::{alloc, Layout};
use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;
use crate::interpreter::{FunctionInterpreter, FunctionInterpreterImpl};
use crate::interpreter::allocation::Value;
use crate::program::global::FunctionImplementation;


pub fn compile_function(function: &FunctionImplementation) -> FunctionInterpreterImpl {
    // Make it our own because the function implementation change at any point.
    let function = Rc::new(function.clone());

    Rc::new(move |interpreter, expression_id, binding| {
        let f = Rc::clone(&function);

        unsafe {
            let arguments = interpreter.evaluate_arguments(expression_id);

            let mut sub_interpreter = FunctionInterpreter {
                implementation: f,
                requirements_fulfillment: FunctionInterpreter::combine_bindings(&interpreter.requirements_fulfillment, binding),
                runtime: interpreter.runtime,
                locals: HashMap::new(),
            };
            sub_interpreter.assign_arguments(arguments);
            sub_interpreter.run()
        }
    })
}

pub fn make_function_getter(function_id: Uuid) -> FunctionInterpreterImpl {
    Rc::new(move |interpreter, expression_id, binding| {
        let fn_layout = Layout::new::<Uuid>();
        unsafe {
            let ptr = alloc(fn_layout);
            *(ptr as *mut Uuid) = function_id;
            Some(Value {
                layout: fn_layout,
                data: ptr,
            })
        }
    })
}
