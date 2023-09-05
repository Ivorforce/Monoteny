use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use uuid::Uuid;
use std::rc::Rc;
use guard::guard;
use crate::linker::precedence::PrecedenceGroup;
use crate::LinkError;
use crate::program::functions::{FunctionHead, FunctionOverload};
use crate::program::traits::Trait;
use crate::program::types::{TypeProto, TypeUnit};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Mutability {
    Immutable,
    Mutable,
}

#[derive(Clone, Eq)]
pub struct Reference {
    pub id: Uuid,
    pub type_: ReferenceType,
}

#[derive(Clone, PartialEq, Eq)]
pub enum ReferenceType {
    Object(Rc<ObjectReference>),
    // Keywords aren't really objects and can't be logically passed around.
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

impl PartialEq for Reference {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Reference {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Reference {
    pub fn make(type_: ReferenceType) -> Rc<Reference> {
        Rc::new(Reference {
            id: Uuid::new_v4(),
            type_,
        })
    }

    pub fn as_object_ref(&self, require_mutable: bool) -> Result<&Rc<ObjectReference>, LinkError> {
        guard!(let ReferenceType::Object(obj_ref) = &self.type_ else {
            return Err(LinkError::LinkError { msg: format!("Reference is not to an object: {:?}", &self.type_) });
        });

        Ok(&obj_ref)
    }

    pub fn as_metatype(&self) -> Result<&TypeUnit, LinkError> {
        let type_ = &self.as_object_ref(false)?.type_;

        guard!(let TypeUnit::MetaType = &type_.unit else {
           return Err(LinkError::LinkError { msg: format!("Reference is not a type.") });
        });

        Ok(&type_.arguments.get(0).unwrap().unit)
    }

    pub fn as_trait(&self) -> Result<Rc<Trait>, LinkError> {
        let type_ = &self.as_object_ref(false)?.type_;

        match type_.unit {
            TypeUnit::MetaType => match &type_.arguments[0].unit {
                TypeUnit::Struct(t) => Ok(Rc::clone(t)),
                _ => Err(LinkError::LinkError { msg: format!("Reference is not a struct metatype.") })
            },
            _ => Err(LinkError::LinkError { msg: format!("Reference is not a metatype.") })
        }
    }

    pub fn as_function_overload(&self) -> Result<Rc<FunctionOverload>, LinkError> {
        match &self.type_ {
            ReferenceType::FunctionOverload(overload) => Ok(Rc::clone(overload)),
            _ => Err(LinkError::LinkError { msg: format!("Reference is not a function in this context.") })
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

    pub fn as_function_head(&self) -> Result<&Rc<FunctionHead>, LinkError> {
        match &self.type_.unit {
            TypeUnit::Function(f) => Ok(f),
            _ => Err(LinkError::LinkError { msg: format!("Object is not a function in this context.") })
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

impl Debug for ReferenceType {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        use ReferenceType::*;
        match self {
            Object(t) => write!(fmt, "{:?}", t.type_),
            FunctionOverload(f) => write!(fmt, "{}", &f.name),
            PrecedenceGroup(p) => write!(fmt, "{}", &p.name),
            Keyword(s) => write!(fmt, "{}", s),
        }
    }
}
