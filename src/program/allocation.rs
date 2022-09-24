use std::hash::{Hash, Hasher};
use uuid::Uuid;
use std::rc::Rc;
use crate::program::types::Type;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Mutability {
    Immutable,
    Mutable,
}

#[derive(Clone)]
pub struct Variable {
    pub id: Uuid,
    pub type_declaration: Box<Type>,
    pub mutability: Mutability
}

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Variable {}

impl Hash for Variable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Variable {
    pub fn make_immutable(type_declaration: Box<Type>) -> Rc<Variable> {
        Rc::new(Variable {
            id: Uuid::new_v4(),
            type_declaration,
            mutability: Mutability::Immutable
        })
    }
}
