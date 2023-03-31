use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;
use crate::interpreter::{FunctionInterpreter, FunctionInterpreterImpl};
use crate::program::global::FunctionImplementation;


pub fn compile_function(function: &Rc<FunctionImplementation>) -> FunctionInterpreterImpl {
    let function = Rc::clone(function);

    Rc::new(move |interpreter, expression_id, binding| {
        let f = Rc::clone(&function);

        unsafe {
            let arguments = interpreter.evaluate_arguments(expression_id);

            let mut sub_interpreter = FunctionInterpreter {
                implementation: &f,
                resolution: FunctionInterpreter::combine_bindings(&interpreter.resolution, binding),
                globals: interpreter.globals,
                assignments: HashMap::new(),
            };
            sub_interpreter.assign_arguments(arguments);
            sub_interpreter.run()
        }
    })
}
