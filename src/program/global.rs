use std::rc::Rc;
use uuid::Uuid;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use crate::program::computation_tree::{ExpressionForest, Statement};
use crate::program::functions::FunctionHead;
use crate::program::allocation::ObjectReference;
use crate::program::generics::TypeForest;
use crate::program::primitives;
use crate::program::traits::RequirementsAssumption;

pub struct FunctionImplementation {
    pub function_id: Uuid,
    pub head: Rc<FunctionHead>,
    pub decorators: Vec<String>,

    pub requirements_assumption: Box<RequirementsAssumption>,

    pub statements: Vec<Box<Statement>>,
    pub expression_forest: Box<ExpressionForest>,
    pub type_forest: Box<TypeForest>,

    pub parameter_variables: Vec<Rc<ObjectReference>>,
    pub variable_names: HashMap<Rc<ObjectReference>, String>,
}

#[derive(Clone, PartialEq, Eq)]
pub enum BuiltinFunctionHint {
    PrimitiveOperation { operation: PrimitiveOperation, type_: primitives::Type },
    Constructor,
    True,
    False,
    Print,
    Panic,
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
    ParseFloatString,
}

impl PartialEq for FunctionImplementation {
    fn eq(&self, other: &Self) -> bool {
        self.function_id == other.function_id
    }
}

impl Eq for FunctionImplementation {}

impl Hash for FunctionImplementation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.function_id.hash(state);
    }
}
