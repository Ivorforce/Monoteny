use std::rc::Rc;
use std::collections::HashMap;
use std::hash::Hash;
use crate::error::{RResult, RuntimeError};
use crate::program::expression_tree::ExpressionTree;
use crate::program::functions::FunctionHead;
use crate::program::allocation::ObjectReference;
use crate::program::generics::TypeForest;
use crate::program::primitives;
use crate::program::traits::RequirementsAssumption;
use crate::source::StructInfo;

#[derive(Clone)]
pub enum FunctionLogic {
    Implementation(Box<FunctionImplementation>),
    Descriptor(FunctionLogicDescriptor),
}

#[derive(Clone)]
pub struct FunctionImplementation {
    pub head: Rc<FunctionHead>,

    pub requirements_assumption: Box<RequirementsAssumption>,

    pub expression_tree: Box<ExpressionTree>,
    pub type_forest: Box<TypeForest>,

    pub parameter_locals: Vec<Rc<ObjectReference>>,
    pub locals_names: HashMap<Rc<ObjectReference>, String>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FunctionLogicDescriptor {
    /// This function was not described by the implementer and is excpected not to be called,
    ///  or to be injected by a transpiler.
    Stub,
    PrimitiveOperation { operation: PrimitiveOperation, type_: primitives::Type },
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
            FunctionLogic::Descriptor(_) => Err(RuntimeError::new(format!("Cannot use a function with described logic as an implementation."))),
        }
    }

    pub fn as_implementation(&self) -> RResult<&FunctionImplementation> {
        match self {
            FunctionLogic::Implementation(i) => Ok(i),
            FunctionLogic::Descriptor(_) => Err(RuntimeError::new(format!("Cannot use a function with described logic as an implementation."))),
        }
    }
}
