use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::zip;
use std::rc::Rc;
use guard::guard;
use uuid::Uuid;
use crate::program::types::{FunctionInterface, Mutability, ParameterKey, PassedArgumentType, Type, Variable};

use crate::program::builtins::TenLangBuiltins;
use crate::program::primitives;

// ================================ Global ==============================

pub struct Program {
    pub functions: Vec<Rc<Function>>,
}

pub struct Function {
    pub interface: Rc<FunctionInterface>,
    pub statements: Vec<Box<Statement>>,
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
    StaticFunctionCall { function: Rc<FunctionInterface>, arguments: Vec<Box<PassedArgument>> },
    PairwiseOperations { arguments: Vec<Box<Expression>>, functions: Vec<Rc<FunctionInterface>> },
    MemberLookup(Box<Expression>, String),
    VariableLookup(Rc<Variable>),
    StringLiteral(String),
    ArrayLiteral(Vec<Box<Expression>>),
}

pub struct PassedArgument {
    pub key: ParameterKey,
    pub value: Box<Expression>,
}

// Impl

impl PassedArgument {
    pub fn to_argument_type(&self) -> PassedArgumentType {
        PassedArgumentType {
            key: self.key.clone(),
            value_type: &self.value.result_type
        }
    }
}
