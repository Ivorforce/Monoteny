use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use uuid::Uuid;
use itertools::zip_eq;
use strum::IntoEnumIterator;
use crate::linker::scopes;
use crate::linker::scopes::{Environment, Scope};
use crate::program::allocation::Reference;
use crate::program::builtins::traits;
use crate::program::builtins::traits::{make_trait, Traits};
use crate::program::functions::{AbstractFunction, FunctionInterface, FunctionPointer};
use crate::program::primitives;
use crate::program::primitives::Type;
use crate::program::traits::{Trait, TraitConformanceDeclaration, TraitConformanceRequirement};
use crate::program::types::{TypeProto, TypeUnit};


pub struct Transpilation {
    pub Transpiler: Rc<Trait>,
    pub add: Rc<FunctionPointer>,
}

pub fn make(mut constants: &mut Scope, traits: &Traits) -> Transpilation {
    let self_id = Uuid::new_v4();
    let self_type = TypeProto::unit(TypeUnit::Any(self_id));

    let Transpiler = make_trait("Transpiler", &self_id, vec![], vec![]);
    constants.insert_trait(&Transpiler);

    let add = FunctionPointer::new_static(FunctionInterface::new_member(
        "add",
        [
            TypeProto::unit(TypeUnit::Struct(Rc::clone(&Transpiler))),
            TypeProto::unit(TypeUnit::Primitive(Type::Int8))  // TODO This should be a function reference
        ].into_iter(),
        self_type.clone(),
    ));
    constants.overload_function(&add);

    Transpilation {
        Transpiler, add
    }
}
