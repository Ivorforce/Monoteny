use std::rc::Rc;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use crate::program::computation_tree::{ExpressionTree, ExpressionID};
use crate::program::functions::FunctionHead;
use crate::program::allocation::ObjectReference;
use crate::program::generics::TypeForest;
use crate::program::primitives;
use crate::program::traits::{RequirementsAssumption, Trait};

#[derive(Clone)]
pub struct FunctionImplementation {
    pub head: Rc<FunctionHead>,

    pub requirements_assumption: Box<RequirementsAssumption>,

    pub root_expression_id: ExpressionID,
    pub expression_forest: Box<ExpressionTree>,
    pub type_forest: Box<TypeForest>,

    pub parameter_locals: Vec<Rc<ObjectReference>>,
    pub locals_names: HashMap<Rc<ObjectReference>, String>,
}

#[derive(Clone, PartialEq, Eq)]
pub enum BuiltinFunctionHint {
    PrimitiveOperation { operation: PrimitiveOperation, type_: primitives::Type },
    Constructor(Rc<Trait>, Vec<Rc<ObjectReference>>),
    GetMemberField(Rc<Trait>, Rc<ObjectReference>),
    SetMemberField(Rc<Trait>, Rc<ObjectReference>),
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
