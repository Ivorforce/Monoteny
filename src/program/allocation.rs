use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use uuid::Uuid;
use std::rc::Rc;
use guard::guard;
use crate::error::{RResult, RuntimeError};
use crate::program::function_object::FunctionOverload;
use crate::program::functions::FunctionHead;
use crate::program::types::{TypeProto, TypeUnit};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Mutability {
    Immutable,
    Mutable,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Reference {
    Local(Rc<ObjectReference>),
    // Keywords aren't really objects and can't be logically passed around.
    // They aren't technically language keywords, but instead were defined in patterns.
    // Yes, this implementation means they can be shadowed!
    Keyword(String),
    // This COULD be an object, but only if it 'inherits' the callable interfaces
    //  from ALL included overloads. Overall, this is probably too confusing and thus not worth
    //  the effort. Rather, as in other languages, we should expect the user to resolve the overload
    //  - either immediately, or by context (e.g. `(should_add ? add : sub)(1, 2)`).
    FunctionOverload(Rc<FunctionOverload>),
}

#[derive(Clone, Eq)]
pub struct ObjectReference {
    pub id: Uuid,
    pub type_: Box<TypeProto>,
    pub mutability: Mutability,
}

impl Reference {
    pub fn as_local(&self, require_mutable: bool) -> RResult<&Rc<ObjectReference>> {
        guard!(let Reference::Local(obj_ref) = self else {
            return Err(RuntimeError::new(format!("Reference is not a local.")));
        });

        Ok(&obj_ref)
    }

    pub fn as_function_overload(&self) -> RResult<Rc<FunctionOverload>> {
        match self {
            Reference::FunctionOverload(overload) => Ok(Rc::clone(overload)),
            _ => Err(RuntimeError::new(format!("Reference is not a function.")))
        }
    }
}

impl ObjectReference {
    pub fn new_immutable(type_: Box<TypeProto>) -> Rc<ObjectReference> {
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

impl Debug for Reference {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Reference::Local(t) => write!(fmt, "{:?}", t.type_),
            Reference::FunctionOverload(f) => write!(fmt, "{}", &f.representation.name),
            Reference::Keyword(s) => write!(fmt, "{}", s),
        }
    }
}
