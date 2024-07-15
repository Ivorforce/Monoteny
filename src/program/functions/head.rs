use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use display_with_options::with_options;
use uuid::Uuid;

use crate::program::functions::{FunctionInterface, FunctionRepresentation};
use crate::program::traits::TraitBinding;

#[derive(Clone, PartialEq, Eq)]
pub enum FunctionType {
    /// A normal function.
    Static,
    /// Not a real function; instead, it refers to a function of some requirement.
    Polymorphic { assumed_requirement: Rc<TraitBinding>, abstract_function: Rc<FunctionHead> },
}

/// The 'head' of a function. It is identifiable by its ID and has an interface.
/// Could be abstract or implemented, depending on whether an implementation is provided!
/// It can also be polymorphic depending on the function_type.
pub struct FunctionHead {
    pub function_id: Uuid,
    pub function_type: FunctionType,
    pub interface: Rc<FunctionInterface>,
    pub declared_representation: FunctionRepresentation,
}

impl FunctionHead {
    pub fn new_static(interface: Rc<FunctionInterface>, declared_representation: FunctionRepresentation) -> Rc<FunctionHead> {
        Self::new(interface, FunctionType::Static, declared_representation)
    }

    pub fn new(interface: Rc<FunctionInterface>, function_type: FunctionType, declared_representation: FunctionRepresentation) -> Rc<FunctionHead> {
        Rc::new(FunctionHead {
            function_id: Uuid::new_v4(),
            interface,
            function_type,
            declared_representation
        })
    }

    pub fn unwrap_id(&self) -> Uuid {
        match &self.function_type {
            FunctionType::Static => self.function_id,
            FunctionType::Polymorphic { .. } => panic!("Cannot unwrap polymorphic implementation ID"),
        }
    }
}

impl PartialEq for FunctionHead {
    fn eq(&self, other: &Self) -> bool {
        self.function_id == other.function_id
    }
}

impl Eq for FunctionHead {}

impl Hash for FunctionHead {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.function_id.hash(state);
    }
}

impl Debug for FunctionHead {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        let call_type_symbol = match self.function_type {
            FunctionType::Static => "|",
            FunctionType::Polymorphic { .. } => "?"
        };
        write!(fmt, "-{}({})--> {:?}", call_type_symbol, &self.function_id, with_options(self.interface.as_ref(), &self.declared_representation))
    }
}
