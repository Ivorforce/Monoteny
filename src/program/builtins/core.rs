use uuid::Uuid;
use std::rc::Rc;
use std::collections::{HashMap, HashSet};
use crate::linker::scopes::Scope;
use crate::program::builtins::primitives::{PrimitiveFunctions};
use crate::program::functions::{Function, FunctionInterface, FunctionPointer};
use crate::program::module::Module;
use crate::program::{builtins, primitives};
use crate::program::builtins::traits::Traits;
use crate::program::traits::Trait;
use crate::program::types::{TypeProto, TypeUnit};

#[allow(non_snake_case)]
pub struct Core {
    pub module: Rc<Module>,
    pub self_id: Uuid,

    pub primitives: HashMap<primitives::Type, Rc<Trait>>,
    pub traits: Traits,
    pub primitive_fns: PrimitiveFunctions,
}

pub fn create() -> Core {
    let mut module = Module::new("monoteny.core".into());

    let self_id = Uuid::new_v4();
    let primitive_traits = builtins::primitives::create_traits(&mut module);
    let traits = builtins::traits::create(&mut module, &primitive_traits);

    Core {
        primitive_fns: builtins::primitives::create_functions(&mut module, &traits, &primitive_traits),

        module: Rc::new(module),
        primitives: primitive_traits,
        traits,
        self_id,
    }
}

impl Core {
    pub fn get_primitive(&self, trait_: &Trait) -> Option<&primitives::Type> {
        for (primitive_type, t) in self.primitives.iter() {
            if trait_.id == t.id {
                return Some(primitive_type)
            }
        }

        None
    }
}
