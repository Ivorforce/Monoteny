use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use uuid::Uuid;
use itertools::zip_eq;
use strum::IntoEnumIterator;
use crate::linker::scopes;
use crate::linker::scopes::{Environment, Scope};
use crate::program::allocation::Reference;
use crate::program::builtins::core;
use crate::program::builtins::core::Core;
use crate::program::builtins::traits::make_trait;
use crate::program::functions::{Function, FunctionCallType, FunctionInterface, FunctionPointer};
use crate::program::module::Module;
use crate::program::primitives;
use crate::program::traits::{Trait, TraitConformanceDeclaration, TraitConformanceRequirement};
use crate::program::types::{TypeProto, TypeUnit};


pub struct Transpilation {
    pub module: Rc<Module>,
    pub any_type: Box<TypeProto>,

    pub Transpiler: Rc<Trait>,
    pub add: Rc<FunctionPointer>,
}

pub fn create(core: &Core) -> Transpilation {
    let mut module = Module::new("monoteny.transpiler".into());

    let self_id = Uuid::new_v4();
    let self_type = TypeProto::unit(TypeUnit::Any(self_id));
    let any_type = TypeProto::unit(TypeUnit::Any(Uuid::new_v4()));

    let Transpiler = make_trait("Transpiler", &self_id, vec![], vec![]);
    module.add_trait(&Transpiler);

    let add = FunctionPointer::new_member(
        "add",
        FunctionInterface::new_simple(
            [
                TypeProto::unit(TypeUnit::Struct(Rc::clone(&Transpiler))),
                // TODO This should be 'any function' but there's no need to implement that until we
                //  need function abstractions in the future otherwise.
                any_type.clone()
            ].into_iter(),
            TypeProto::unit(TypeUnit::Void),
        )
    );
    module.add_function(&add);

    Transpilation {
        module: Rc::new(module),
        any_type,

        Transpiler, add
    }
}
