use std::alloc::{alloc, Layout};
use uuid::Uuid;
use crate::interpreter::{compiler, InterpreterGlobals, Value};
use crate::program::module::Module;


pub fn module(module: &Module, globals: &mut InterpreterGlobals) {
    for (function_pointer, implementation) in module.function_implementations.iter() {
        globals.function_evaluators.insert(implementation.head.function_id.clone(), compiler::compile_function(implementation));

        unsafe {
            let fn_layout = Layout::new::<Uuid>();
            let ptr = alloc(fn_layout);
            *(ptr as *mut Uuid) = implementation.implementation_id;
            globals.assignments.insert(
                module.functions_references[function_pointer].id,
                Value { data: ptr, layout: fn_layout }
            );
        }
    }
}
