use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use uuid::Uuid;
use crate::abstract_syntax::Mutability;

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
    Name(String),
    Int(i32),
}

// ================================ Type ==============================

#[derive(Clone)]
pub struct Variable {
    pub id: Uuid,
    pub name: String,
    pub type_declaration: Box<Type>,
    pub mutability: Mutability,
}

#[derive(Clone, PartialEq)]
pub enum Type {
    Primitive(PrimitiveType),
    Identifier(String),
    NDArray(Box<Type>),
    Function(Rc<FunctionInterface>),
    Generic(Uuid),
}

#[derive(Copy, Clone, PartialEq)]
pub enum Primitive {
    Bool(bool),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Int128(i128),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    UInt128(u128),
    Float32(f32),
    Float64(f64),
}

#[derive(Copy, Clone, PartialEq)]
pub enum PrimitiveType {
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    Float32,
    Float64,
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

impl Primitive {
    pub fn primitive_type(&self) -> PrimitiveType {
        use Primitive::*;
        match self {
            Bool(_) => PrimitiveType::Bool,
            Int8(_) => PrimitiveType::Int8,
            Int16(_) => PrimitiveType::Int16,
            Int32(_) => PrimitiveType::Int32,
            Int64(_) => PrimitiveType::Int64,
            Int128(_) => PrimitiveType::Int128,
            UInt8(_) => PrimitiveType::UInt8,
            UInt16(_) => PrimitiveType::UInt16,
            UInt32(_) => PrimitiveType::UInt32,
            UInt64(_) => PrimitiveType::UInt64,
            UInt128(_) => PrimitiveType::UInt128,
            Float32(_) => PrimitiveType::Float32,
            Float64(_) => PrimitiveType::Float64,
        }
    }
}


impl PrimitiveType {
    pub fn identifier_string(&self) -> String {
        use PrimitiveType::*;
        String::from(match self {
            Bool => "Bool",
            Int8 => "Int8",
            Int16 => "Int16",
            Int32 => "Int32",
            Int64 => "Int64",
            Int128 => "Int128",
            UInt8 => "UInt8",
            UInt16 => "UInt16",
            UInt32 => "UInt32",
            UInt64 => "UInt64",
            UInt128 => "UInt128",
            Float32 => "Float32",
            Float64 => "Float64",
        })
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
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
