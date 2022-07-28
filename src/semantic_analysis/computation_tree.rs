use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use uuid::Uuid;

// ================================ Global ==============================

pub struct Program {
    pub functions: Vec<Box<Function>>,
    pub variables: HashMap<Uuid, Rc<Variable>>,
}

pub struct Function {
    pub identifier: String,
    pub parameters: Vec<Box<Parameter>>,
    pub return_type: Option<Box<Type>>,

    pub variables: HashMap<Uuid, Rc<Variable>>,
    pub statements: Vec<Box<Statement>>,
}

pub struct Parameter {
    pub external_name: String,
    pub variable: Rc<Variable>
}

// ================================ Type ==============================

#[derive(Clone)]
pub struct Variable {
    pub id: Uuid,
    pub home: VariableHome,
    pub name: String,
    pub type_declaration: Box<Type>,
}

#[derive(Clone, PartialEq)]
pub enum Type {
    Identifier(String),
    NDArray(Box<Type>),
}

// ================================ Code ==============================

#[derive(Copy, Clone)]
pub enum VariableHome {
    Local, Global
}

pub enum Statement {
    VariableAssignment(String, Box<Expression>),
    Expression(Box<Expression>),
    Return(Option<Box<Expression>>),
}

pub struct Expression {
    pub result_type: Box<Type>,
    pub operation: Box<ExpressionOperation>,
}

pub enum ExpressionOperation {
    Number(i32),
    FunctionCall(Box<Expression>, Vec<Box<PassedArgument>>),
    MemberLookup(Box<Expression>, String),
    VariableLookup(Rc<Variable>),
    ArrayLiteral(Vec<Box<Expression>>),
    StringLiteral(String),
}

pub struct PassedArgument {
    pub name: Option<String>,
    pub value: Box<Expression>,
}

// String

impl Debug for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

