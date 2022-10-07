use std::hash::{Hash, Hasher};
use uuid::Uuid;
use std::rc::Rc;
use crate::program::types::TypeProto;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Mutability {
    Immutable,
    Mutable,
}

#[derive(Clone)]
pub struct Reference {
    pub id: Uuid,
    pub type_declaration: Box<TypeProto>,
    pub mutability: Mutability
}

impl PartialEq for Reference {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Reference {}

impl Hash for Reference {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Reference {
    pub fn make_immutable(type_declaration: Box<TypeProto>) -> Rc<Reference> {
        Rc::new(Reference {
            id: Uuid::new_v4(),
            type_declaration,
            mutability: Mutability::Immutable
        })
    }
}
