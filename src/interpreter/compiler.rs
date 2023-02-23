use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;
use crate::interpreter::{FunctionInterpreter, FunctionInterpreterImpl};
use crate::program::global::FunctionImplementation;

pub fn compile_function(function: &Rc<FunctionImplementation>) -> FunctionInterpreterImpl {
    let function = Rc::clone(function);

    Box::new(move |interpreter, expression_id, binding| {
        unsafe {
            let arguments = interpreter.evaluate_arguments(expression_id);

            let mut sub_interpreter = FunctionInterpreter {
                builtins: &interpreter.builtins,
                function_evaluators: &interpreter.function_evaluators,
                implementation: &function,
                binding: FunctionInterpreter::combine_bindings(&interpreter.binding, binding),
                assignments: HashMap::new(),
            };
            sub_interpreter.assign_arguments(arguments);
            sub_interpreter.run()
        }
    })
}
