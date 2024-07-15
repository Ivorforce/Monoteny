use std::rc::Rc;

use crate::error::{RResult, RuntimeError};
use crate::program::allocation::ObjectReference;
use crate::program::functions::{FunctionHead, FunctionImplementation};
use crate::program::primitives;
use crate::program::traits::{StructInfo, Trait};
use crate::program::types::TypeProto;

#[derive(Clone)]
pub enum FunctionLogic {
    Implementation(Box<FunctionImplementation>),
    Descriptor(FunctionLogicDescriptor),
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FunctionLogicDescriptor {
    /// This function was not described by the implementer and is expected not to be called,
    ///  or to be injected by a transpiler.
    Stub,
    TraitProvider(Rc<Trait>),
    FunctionProvider(Rc<FunctionHead>),
    PrimitiveOperation { operation: PrimitiveOperation, type_: primitives::Type },
    Clone(Rc<TypeProto>),
    Constructor(Rc<StructInfo>),
    GetMemberField(Rc<StructInfo>, Rc<ObjectReference>),
    SetMemberField(Rc<StructInfo>, Rc<ObjectReference>),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PrimitiveOperation {
    And, Or, Not,
    Negative,
    Add, Subtract,
    Multiply, Divide,
    Modulo,
    Exp, Log,
    EqualTo, NotEqualTo,
    GreaterThan, LesserThan,
    GreaterThanOrEqual, LesserThanOrEqual,
    ParseIntString,
    ParseRealString,
    ToString,
    Clone,
}

impl FunctionLogic {
    pub fn is_implementation(&self) -> bool {
        match self {
            FunctionLogic::Implementation(_) => true,
            FunctionLogic::Descriptor(_) => false,
        }
    }

    pub fn to_implementation(self) -> RResult<Box<FunctionImplementation>> {
        match self {
            FunctionLogic::Implementation(i) => Ok(i),
            FunctionLogic::Descriptor(_) => Err(RuntimeError::error("Cannot use a function with described logic as an implementation.").to_array()),
        }
    }

    pub fn as_implementation(&self) -> RResult<&FunctionImplementation> {
        match self {
            FunctionLogic::Implementation(i) => Ok(i),
            FunctionLogic::Descriptor(_) => Err(RuntimeError::error("Cannot use a function with described logic as an implementation.").to_array()),
        }
    }
}
