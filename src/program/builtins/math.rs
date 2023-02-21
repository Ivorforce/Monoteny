use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use uuid::Uuid;
use itertools::zip_eq;
use strum::IntoEnumIterator;
use crate::program::allocation::Reference;
use crate::program::builtins::core;
use crate::program::builtins::core::Core;
use crate::program::functions::{FunctionInterface, FunctionPointer};
use crate::program::module::Module;
use crate::program::primitives;
use crate::program::traits::{Trait, TraitConformanceDeclaration, TraitConformanceRequirement};
use crate::program::types::{TypeProto, TypeUnit};


// TODO This module should be written in monoteny.
pub struct Math {
    pub module: Rc<Module>,
    pub pi: Rc<FunctionPointer>,
    pub tau: Rc<FunctionPointer>,
    pub e: Rc<FunctionPointer>,
}


pub fn create(core: &Core) -> Math {
    let mut module = Module::new("monoteny.math".into());

    let float_generic = TypeProto::make_any();
    let float_requirement = Rc::new(TraitConformanceRequirement {
        id: Uuid::new_v4(),
        trait_: Rc::clone(&core.traits.Float),
        binding: HashMap::from(([(*core.traits.Float.generics.iter().next().unwrap(), float_generic.clone())]))
    });

    // TODO We should also provide builtin implementations for these (call to from_literal)

    let pi = FunctionPointer::new_static(FunctionInterface::new_constant("pi", &float_generic, vec![&float_requirement]));
    module.add_function(&pi);

    let tau = FunctionPointer::new_static(FunctionInterface::new_constant("tau", &float_generic, vec![&float_requirement]));
    module.add_function(&tau);

    let e = FunctionPointer::new_static(FunctionInterface::new_constant("e", &float_generic, vec![&float_requirement]));
    module.add_function(&e);

    Math {
        module: Rc::new(module),
        pi, tau, e,
    }
}
