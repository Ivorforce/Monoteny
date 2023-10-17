use std::collections::HashMap;
use std::rc::Rc;
use crate::interpreter::Runtime;
use crate::linker::scopes;
use crate::program;
use crate::program::module::{Module, module_name};
use crate::program::traits::Trait;

pub mod primitives;
pub mod traits;

pub fn create_builtins(runtime: &mut Runtime) -> Box<Module> {
    let mut module = Box::new(Module::new(module_name("builtins")));

    runtime.primitives = Some(primitives::create_traits(runtime, &mut module));
    runtime.traits = Some(traits::create(runtime, &mut module));
    primitives::create_functions(runtime, &mut module);
    module
}
