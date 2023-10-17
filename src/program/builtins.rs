use std::collections::HashMap;
use std::rc::Rc;
use crate::linker::scopes;
use crate::program;
use crate::program::module::{Module, module_name};
use crate::program::traits::Trait;

pub mod primitives;
pub mod traits;

#[allow(non_snake_case)]
pub struct Builtins {
    pub Metatype: Rc<Trait>,
    pub primitives: HashMap<program::primitives::Type, Rc<Trait>>,
    pub traits: traits::Traits,
}

pub fn create_builtins(module: &mut Module) -> Rc<Builtins> {
    let mut Metatype = Trait::new_with_self("Type".to_string());
    let Metatype = Rc::new(Metatype);
    module.add_trait(&Metatype, &Metatype);

    let primitive_traits = primitives::create_traits(&Metatype, module);
    let traits = traits::create(module, &Metatype, &primitive_traits);
    primitives::create_functions(module, &traits, &primitive_traits);

    Rc::new(Builtins {
        Metatype,
        primitives: primitive_traits,
        traits,
    })
}

impl Builtins {
    pub fn get_primitive(&self, trait_: &Trait) -> Option<&program::primitives::Type> {
        for (primitive_type, t) in self.primitives.iter() {
            if trait_.id == t.id {
                return Some(primitive_type)
            }
        }

        None
    }
}
