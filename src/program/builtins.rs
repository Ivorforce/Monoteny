use std::rc::Rc;

use crate::linker::scopes::Scope;

pub mod precedence;
pub mod debug;
pub mod traits;
pub mod primitives;
pub mod math;
pub mod common;
mod transpilation;

pub struct Builtins {
    pub traits: traits::Traits,
    pub debug: debug::Functions,
    pub primitives: primitives::Primitives,
    pub precedence_groups: precedence::PrecedenceGroups,

    pub common: common::Common,
    pub math: math::Math,
    pub transpilation: transpilation::Transpilation,

    pub global_constants: Scope<'static>,
}

pub fn create_builtins() -> Rc<Builtins> {
    let mut constants: Scope = Scope::new();

    let precedence_groups = precedence::make_groups(&mut constants);
    let traits = traits::make(&mut constants);
    let primitives = primitives::make(&mut constants, &traits);

    Rc::new(Builtins {
        common: common::make(&mut constants, &traits),
        math: math::make(&mut constants, &traits),
        debug: debug::make_functions(&mut constants),
        transpilation: transpilation::make(&mut constants, &traits),
        primitives,
        precedence_groups,
        traits,
        global_constants: constants,
    })
}
