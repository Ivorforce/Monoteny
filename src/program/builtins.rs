use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use itertools::zip_eq;
use uuid::Uuid;
use precedence::PrecedenceGroups;
use primitives::Primitives;
use traits::Traits;
use crate::program::traits::{Trait, TraitConformanceDeclaration, TraitConformanceRequirement};
use crate::parser;
use crate::linker::precedence::{OperatorAssociativity, PrecedenceGroup};
use crate::linker::scopes::Scope;
use crate::program::types::*;
use crate::program;
use crate::program::allocation::Reference;
use crate::program::functions::{FunctionForm, FunctionPointer, HumanFunctionInterface, MachineFunctionInterface};
use crate::program::structs::Struct;

pub mod precedence;
pub mod debug;
pub mod strings;
pub mod traits;
pub mod primitives;

pub struct Builtins {
    pub traits: Traits,
    pub debug: debug::Functions,
    pub primitives: Primitives,
    pub strings: strings::Strings,
    pub precedence_groups: PrecedenceGroups,

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
