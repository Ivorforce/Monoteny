use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use uuid::Uuid;
use std::rc::Rc;
use crate::program::types::TypeProto;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Mutability {
    Immutable,
    Mutable,
}

#[derive(Clone, Eq)]
pub struct ObjectReference {
    pub id: Uuid,
    pub type_: Rc<TypeProto>,
    pub mutability: Mutability,
}

impl ObjectReference {
    pub fn new_immutable(type_: Rc<TypeProto>) -> Rc<ObjectReference> {
        Rc::new(ObjectReference {
            id: Uuid::new_v4(),
            type_,
            mutability: Mutability::Immutable
        })
    }
}

impl PartialEq for ObjectReference {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for ObjectReference {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Debug for ObjectReference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut_keyword = match self.mutability {
            Mutability::Immutable => "let",
            Mutability::Mutable => "var",
        };
        write!(f, "{} <{}> '{:?}", mut_keyword, self.id, self.type_)
    }
}
