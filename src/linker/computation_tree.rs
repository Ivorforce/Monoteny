use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::zip;
use std::rc::Rc;
use guard::guard;
use uuid::Uuid;
use crate::program::types::{Mutability, ParameterKey, Type, Variable};

use crate::program::builtins::TenLangBuiltins;
use crate::program::functions::HumanFunctionInterface;
use crate::program::primitives;
use crate::program::traits::TraitBinding;

// ================================ Global ==============================

pub struct Program {
    pub functions: Vec<Rc<FunctionImplementation>>,
}

pub struct FunctionImplementation {
    pub interface: Rc<HumanFunctionInterface>,
    pub statements: Vec<Box<Statement>>,
    pub variable_names: HashMap<Rc<Variable>, String>
}

// ================================ Code ==============================

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
    FunctionCall { function: Rc<HumanFunctionInterface>, arguments: HashMap<Rc<Variable>, Box<Expression>>, binding: Box<TraitBinding> },
    PairwiseOperations { arguments: Vec<Box<Expression>>, functions: Vec<Rc<HumanFunctionInterface>> },
    MemberLookup(Box<Expression>, String),
    VariableLookup(Rc<Variable>),
    StringLiteral(String),
    ArrayLiteral(Vec<Box<Expression>>),
}
