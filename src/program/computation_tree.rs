use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use guard::guard;
use uuid::Uuid;
use crate::parser::abstract_syntax::Function;
use crate::program::allocation::{Mutability, Variable};
use crate::program::types::Type;

use crate::program::builtins::TenLangBuiltins;
use crate::program::functions::{FunctionPointer, HumanFunctionInterface, MachineFunctionInterface, ParameterKey};
use crate::program::primitives;
use crate::program::traits::{Trait, TraitBinding, TraitConformanceDeclaration, TraitConformanceRequirement};

pub enum Statement {
    VariableAssignment(Rc<Variable>, Box<Expression>),
    Expression(Box<Expression>),
    Return(Option<Box<Expression>>),
}

pub struct Expression {
    pub result_type: Option<Box<Type>>,
    pub operation: Box<ExpressionOperation>,
}

pub enum ExpressionOperation {
    Primitive(primitives::Value),
    FunctionCall { function: Rc<FunctionPointer>, arguments: HashMap<Rc<Variable>, Box<Expression>>, binding: Box<TraitBinding> },
    PairwiseOperations { arguments: Vec<Box<Expression>>, functions: Vec<Rc<HumanFunctionInterface>> },
    MemberLookup(Box<Expression>, String),
    VariableLookup(Rc<Variable>),
    StringLiteral(String),
    ArrayLiteral(Vec<Box<Expression>>),
}
