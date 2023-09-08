use std::collections::HashMap;
use std::rc::Rc;
use crate::interpreter::{FunctionInterpreter, FunctionInterpreterImpl};
use crate::program::global::FunctionImplementation;


pub fn compile_function(function: &Rc<FunctionImplementation>) -> FunctionInterpreterImpl {
    let function = Rc::clone(function);

    Rc::new(move |interpreter, expression_id, binding| {
        let f = Rc::clone(&function);

        unsafe {
            let arguments = interpreter.evaluate_arguments(expression_id);

            let mut sub_interpreter = FunctionInterpreter {
                implementation: f,
                requirements_fulfillment: FunctionInterpreter::combine_bindings(&interpreter.requirements_fulfillment, binding),
                globals: interpreter.globals,
                locals: HashMap::new(),
            };
            sub_interpreter.assign_arguments(arguments);
            sub_interpreter.run()
        }
    })
}
