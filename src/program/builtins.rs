use std::rc::Rc;

use crate::linker::scopes::Scope;

pub mod precedence;
pub mod debug;
pub mod strings;
pub mod traits;
pub mod primitives;

pub struct Builtins {
    pub traits: traits::Traits,
    pub debug: debug::Functions,
    pub primitives: primitives::Primitives,
    pub strings: strings::Strings,
    pub precedence_groups: precedence::PrecedenceGroups,

    pub global_constants: Scope<'static>,
}

pub fn create_builtins() -> Rc<Builtins> {
    let mut constants: Scope = Scope::new();

    let precedence_groups = precedence::make_groups(&mut constants);
    let traits = traits::make(&mut constants);
    let primitives = primitives::make(&mut constants, &traits);

    Rc::new(Builtins {
        traits,
        primitives,
        debug: debug::make_functions(&mut constants),
        strings: strings::make(&mut constants),
        precedence_groups,
        global_constants: constants,
    })
}
