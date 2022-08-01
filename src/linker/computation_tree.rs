use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use uuid::Uuid;
use crate::abstract_syntax::Mutability;

use crate::linker::builtins::TenLangBuiltins;
use crate::linker::primitives;

// ================================ Global ==============================

pub struct Program {
    pub functions: Vec<Rc<Function>>,
    pub builtins: Rc<TenLangBuiltins>,
}

pub struct FunctionInterface {
    pub id: Uuid,
    pub name: String,

    pub parameters: Vec<Box<Parameter>>,
    pub generics: Vec<Rc<Generic>>,

    pub return_type: Option<Box<Type>>,
}

pub struct Function {
    pub interface: Rc<FunctionInterface>,
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
    Primitive(primitives::Type),
    Identifier(String),
    NDArray(Box<Type>),
    Function(Rc<FunctionInterface>),
    Generic(Rc<Generic>),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Generic {
    pub id: Uuid,
    pub name: String,
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
        use Type::*;
        match self {
            Primitive(p) => write!(fmt, "{}", p.identifier_string()),
            Identifier(i) => write!(fmt, "{}", i),
            NDArray(atom) => write!(fmt, "{:?}", atom),
            Function(f) => write!(fmt, "(?) -> ({:?})", f.return_type),
            Generic(g) => write!(fmt, "{}", g.name),
        }
    }
}

impl Type {
    pub fn collect_generics<'a>(&'a self, set: &mut HashSet<&'a Rc<Generic>>) {
        match self {
            Type::Primitive(_) => {}
            Type::Identifier(_) => {}
            Type::NDArray(atom) => {
                atom.collect_generics(set);
            }
            Type::Function(fun) => {
                if let Some(return_type) = &fun.return_type {
                    return_type.collect_generics(set);
                }
            }
            Type::Generic(generic) => {
                let _ = &set.insert(generic);
            }
        }
    }
}

pub fn get_common_supertype<'a>(types: &Vec<&'a Box<Type>>) -> &'a Box<Type> {
    if types.is_empty() {
        panic!("Empty (inferred) array types are not supported for now.");
    }

    let reference = types[0];
    for other in types.iter().skip(1) {
        if *other != reference {
            panic!("Supertype inferral is not supported yet.")
        }
    }

    return reference;
}
