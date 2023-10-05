use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use uuid::Uuid;
use std::rc::Rc;
use guard::guard;
use crate::error::RuntimeError;
use crate::linker::precedence::PrecedenceGroup;
use crate::program::functions::{FunctionHead, FunctionOverload};
use crate::program::traits::Trait;
use crate::program::types::{TypeProto, TypeUnit};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Mutability {
    Immutable,
    Mutable,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Reference {
    Object(Rc<ObjectReference>),
    // Keywords aren't really objects and can't be logically passed around.
    // They aren't technically language keywords, but instead were defined in patterns.
    // This means they can be shadowed!
    Keyword(String),
    // This COULD be an object, but only if it 'inherits' the callable interfaces
    //  from ALL included overloads. Overall, this is probably too confusing and thus not worth
    //  the effort. Rather, as in other languages, we should expect the user to resolve the overload
    //  - either immediately, or by context (e.g. `(should_add ? add : sub)(1, 2)`).
    FunctionOverload(Rc<FunctionOverload>),
    // Maybe this could be an object or even trait in the future.
    PrecedenceGroup(Rc<PrecedenceGroup>),
}

#[derive(Clone, Eq)]
pub struct ObjectReference {
    pub id: Uuid,
    pub type_: Box<TypeProto>,
    pub mutability: Mutability,
}

impl Reference {
    pub fn as_object_ref(&self, require_mutable: bool) -> Result<&Rc<ObjectReference>, RuntimeError> {
        guard!(let Reference::Object(obj_ref) = self else {
            return Err(RuntimeError::new(format!("Reference is not to an object: {:?}", self)));
        });

        Ok(&obj_ref)
    }

    pub fn as_metatype(&self) -> Result<&TypeUnit, RuntimeError> {
        let type_ = &self.as_object_ref(false)?.type_;

        guard!(let TypeUnit::MetaType = &type_.unit else {
           return Err(RuntimeError::new(format!("Reference is not a type.")));
        });

        Ok(&type_.arguments.get(0).unwrap().unit)
    }

    pub fn as_trait(&self) -> Result<Rc<Trait>, RuntimeError> {
        let type_ = &self.as_object_ref(false)?.type_;

        match type_.unit {
            TypeUnit::MetaType => match &type_.arguments[0].unit {
                TypeUnit::Struct(t) => Ok(Rc::clone(t)),
                _ => Err(RuntimeError::new(format!("Reference is not a struct metatype.")))
            },
            _ => Err(RuntimeError::new(format!("Reference is not a metatype.")))
        }
    }

    pub fn as_function_overload(&self) -> Result<Rc<FunctionOverload>, RuntimeError> {
        match self {
            Reference::FunctionOverload(overload) => Ok(Rc::clone(overload)),
            _ => Err(RuntimeError::new(format!("Reference is not a function in this context.")))
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

    pub fn as_function_head(&self) -> Result<&Rc<FunctionHead>, RuntimeError> {
        match &self.type_.unit {
            TypeUnit::Function(f) => Ok(f),
            _ => Err(RuntimeError::new(format!("Object is not a function in this context.")))
        }
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
            Reference::Object(t) => write!(fmt, "{:?}", t.type_),
            Reference::FunctionOverload(f) => write!(fmt, "{}", &f.name),
            Reference::PrecedenceGroup(p) => write!(fmt, "{}", &p.name),
            Reference::Keyword(s) => write!(fmt, "{}", s),
        }
    }
}
