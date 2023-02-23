use std::alloc::{alloc, Layout};
use std::collections::HashMap;
use uuid::Uuid;
use crate::interpreter::{builtins, compiler, FunctionInterpreter, Value};
use crate::program::builtins::Builtins;
use crate::program::Program;
use crate::program::traits::TraitBinding;


pub fn main(program: &Program, builtins: &Builtins) {
    let main_function = program.find_main().expect("No main function!");
    let mut evaluators = builtins::make_evaluators(builtins);
    let mut assignments = HashMap::new();

    for (function_pointer, implementation) in program.function_implementations.iter() {
        evaluators.insert(implementation.pointer.target.function_id.clone(), compiler::compile_function(implementation));

        unsafe {
            let fn_layout = Layout::new::<Uuid>();
            let ptr = alloc(fn_layout);
            *(ptr as *mut Uuid) = implementation.implementation_id;
            assignments.insert(
                program.module.functions[function_pointer].id,
                Value { data: ptr, layout: fn_layout }
            );
        }
    }

    let mut interpreter = FunctionInterpreter {
        builtins,
        function_evaluators: &evaluators,
        implementation: main_function,
        binding: TraitBinding { resolution: HashMap::new() },
        assignments,
    };
    unsafe {
        interpreter.run();
    }
}
