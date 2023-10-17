use crate::interpreter::Runtime;
use crate::program::module::{Module, module_name};

pub mod primitives;
pub mod traits;

pub fn create_builtins(runtime: &mut Runtime) -> Box<Module> {
    let mut module = Box::new(Module::new(module_name("builtins")));

    runtime.primitives = Some(primitives::create_traits(runtime, &mut module));
    runtime.traits = Some(traits::create(runtime, &mut module));
    primitives::create_functions(runtime, &mut module);
    module
}
