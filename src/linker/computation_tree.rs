use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use uuid::Uuid;

use crate::linker::builtins::TenLangBuiltins;

// ================================ Global ==============================

pub struct Program {
    pub functions: Vec<Rc<Function>>,
    pub variables: HashMap<Uuid, Rc<Variable>>,
    pub builtins: TenLangBuiltins,
}

pub struct FunctionInterface {
    pub id: Uuid,
    pub name: String,
    pub parameters: Vec<Box<Parameter>>,
    pub return_type: Option<Box<Type>>,
}

pub struct Function {
    pub interface: Rc<FunctionInterface>,
    pub variables: HashMap<Uuid, Rc<Variable>>,
    pub statements: Vec<Box<Statement>>,
}

pub struct Parameter {
    pub external_key: ParameterKey,
    pub variable: Rc<Variable>
}

#[derive(Clone)]
pub enum ParameterKey {
    Keyless,
    String(String)
}

// ================================ Type ==============================

#[derive(Clone)]
pub struct Variable {
    pub id: Uuid,
    pub name: String,
    pub type_declaration: Box<Type>,
}

#[derive(Clone, PartialEq)]
pub enum Type {
    Identifier(String),
    NDArray(Box<Type>),
    Function(Rc<FunctionInterface>),
    Generic(Uuid),
}

#[derive(Clone, PartialEq)]
pub enum Primitive {
    Float32(f32),
    Float64(f64),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Int128(i128),
    String(String),
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
    Primitive(Primitive),
    StaticFunctionCall { function: Rc<FunctionInterface>, arguments: Vec<Box<PassedArgument>> },
    DynamicFunctionCall(Box<Expression>, Vec<Box<PassedArgument>>),
    MemberLookup(Box<Expression>, String),
    VariableLookup(Rc<Variable>),
    ArrayLiteral(Vec<Box<Expression>>),
}

pub struct PassedArgument {
    pub key: ParameterKey,
    pub value: Box<Expression>,
}

// Impl

impl Primitive {
    fn native_type(&self) -> Type {
        Type::Identifier(String::from(match self {
            Primitive::Float32(_) => "Float32",
            Primitive::Float64(_) => "Float64",
            Primitive::Int8(_) => "Int8",
            Primitive::Int16(_) => "Int16",
            Primitive::Int32(_) => "Int32",
            Primitive::Int64(_) => "Int64",
            Primitive::Int128(_) => "Int128",
            Primitive::String(_) => "String",
        }))
    }
}

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialEq for FunctionInterface {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Debug for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}