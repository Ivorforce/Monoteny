use std::collections::HashMap;
use std::rc::Rc;
use crate::linker::scopes;
use crate::program::module::Module;

pub mod precedence;
pub mod core;
pub mod primitives;
pub mod common;
pub mod transpilation;
pub mod traits;

pub struct Builtins {
    pub core: core::Core,
    pub precedence_groups: precedence::PrecedenceGroups,
    pub transpilation: transpilation::Transpilation,
}

pub fn create_builtins() -> Rc<Builtins> {
    let core = core::create();
    let mut builtins = Builtins {
        transpilation: transpilation::create(&core),
        core,
        precedence_groups: precedence::create(),
    };

    Rc::new(builtins)
}

impl Builtins {
    pub fn all_modules(&self) -> Vec<&Rc<Module>> {
        vec![
            &self.core.module,
            &self.precedence_groups.module,
            &self.transpilation.module,
        ]
    }

    pub fn create_scope(&self) -> scopes::Scope {
        let mut scope = scopes::Scope::new();

        for precedence_group in self.precedence_groups.list.iter() {
            scope.precedence_groups.push((Rc::clone(precedence_group), HashMap::new()));
        }

        for module in self.all_modules() {
            scope.import(module).unwrap();
        }

        scope
    }
}
