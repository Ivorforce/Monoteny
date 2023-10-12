use std::collections::HashMap;
use std::rc::Rc;
use crate::linker::scopes;
use crate::program;
use crate::program::module::Module;
use crate::program::traits::Trait;

pub mod primitives;
pub mod traits;

pub struct Builtins {
    pub module: Rc<Module>,

    pub primitives: HashMap<program::primitives::Type, Rc<Trait>>,
    pub traits: traits::Traits,
}

pub fn create_builtins() -> Rc<Builtins> {
    let mut module = Module::new("monoteny.core".to_string());

    let primitive_traits = primitives::create_traits(&mut module);
    let traits = traits::create(&mut module, &primitive_traits);
    primitives::create_functions(&mut module, &traits, &primitive_traits);

    Rc::new(Builtins {
        module: Rc::new(module),
        primitives: primitive_traits,
        traits,
    })
}

impl Builtins {
    pub fn create_scope<'a>(&self) -> scopes::Scope<'a> {
        let mut scope = scopes::Scope::new();

        scope.import(&self.module).unwrap();

        scope
    }

    pub fn get_primitive(&self, trait_: &Trait) -> Option<&program::primitives::Type> {
        for (primitive_type, t) in self.primitives.iter() {
            if trait_.id == t.id {
                return Some(primitive_type)
            }
        }

        None
    }
}
