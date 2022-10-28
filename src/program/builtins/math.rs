use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use uuid::Uuid;
use itertools::zip_eq;
use strum::IntoEnumIterator;
use crate::linker::scopes;
use crate::linker::scopes::{Environment, Scope};
use crate::program::allocation::Reference;
use crate::program::builtins::traits;
use crate::program::builtins::traits::Traits;
use crate::program::functions::FunctionPointer;
use crate::program::primitives;
use crate::program::traits::{Trait, TraitConformanceDeclaration, TraitConformanceRequirement};
use crate::program::types::{TypeProto, TypeUnit};


pub struct Math {
    pub pi: Rc<FunctionPointer>,
    pub tau: Rc<FunctionPointer>,
    pub e: Rc<FunctionPointer>,
}


pub fn make(mut constants: &mut Scope, traits: &Traits) -> Math {
    let float_generic = TypeProto::make_any();
    let float_requirement = Rc::new(TraitConformanceRequirement {
        id: Uuid::new_v4(),
        trait_: Rc::clone(&traits.Float),
        arguments: vec![float_generic.clone()]
    });

    // TODO We should also provide builtin implementations for these (call to from_literal)

    let pi = FunctionPointer::make_constant("pi", &float_generic, vec![&float_requirement]);
    constants.overload_function(&pi);

    let tau = FunctionPointer::make_constant("tau", &float_generic, vec![&float_requirement]);
    constants.overload_function(&tau);

    let e = FunctionPointer::make_constant("e", &float_generic, vec![&float_requirement]);
    constants.overload_function(&e);

    Math {
        pi, tau, e,
    }
}
