use std::collections::HashMap;
use std::rc::Rc;
use crate::linker::scopes;

pub mod precedence;
pub mod debug;
pub mod core;
pub mod primitives;
pub mod math;
pub mod common;
pub mod transpilation;
pub mod traits;

pub struct Builtins {
    pub core: core::Core,
    pub precedence_groups: precedence::PrecedenceGroups,

    pub common: common::Common,
    pub math: math::Math,

    pub debug: debug::Debug,
    pub transpilation: transpilation::Transpilation,
}

pub fn create_builtins() -> Rc<Builtins> {
    let core = core::create();

    Rc::new(Builtins {
        common: common::create(&core),
        math: math::create(&core),
        debug: debug::create(),
        transpilation: transpilation::create(&core),
        core,
        precedence_groups: precedence::create(),
    })
}

impl Builtins {
    pub fn create_scope(&self) -> scopes::Scope {
        let mut scope = scopes::Scope::new();

        for precedence_group in self.precedence_groups.list.iter() {
            scope.precedence_groups.push((Rc::clone(precedence_group), HashMap::new()));
        }

        for module in [
            &self.core.module,
            &self.precedence_groups.module,
            &self.common.module,
            &self.math.module,
            &self.debug.module,
            &self.transpilation.module,
        ] {
            scope.import(module).unwrap();
        }

        scope
    }
}
