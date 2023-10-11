use std::collections::HashMap;
use std::rc::Rc;
use crate::linker::scopes;
use crate::program::module::Module;

pub mod core;
pub mod primitives;
pub mod common;
pub mod transpilation;
pub mod traits;

pub struct Builtins {
    pub core: core::Core,
    pub transpilation: transpilation::Transpilation,
}

pub fn create_builtins() -> Rc<Builtins> {
    let core = core::create();
    let mut builtins = Builtins {
        transpilation: transpilation::create(&core),
        core,
    };

    Rc::new(builtins)
}

impl Builtins {
    pub fn all_modules(&self) -> Vec<&Rc<Module>> {
        vec![
            &self.core.module,
            &self.transpilation.module,
        ]
    }

    pub fn create_scope<'a>(&self) -> scopes::Scope<'a> {
        let mut scope = scopes::Scope::new();

        for module in self.all_modules() {
            scope.import(module).unwrap();
        }

        scope
    }
}
