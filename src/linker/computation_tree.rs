use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::zip;
use std::rc::Rc;
use guard::guard;
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

    pub is_member_function: bool,
    pub parameters: Vec<Box<Parameter>>,
    pub generics: Vec<Rc<Generic>>,

    pub return_type: Option<Box<Type>>,
}

pub struct Function {
    pub interface: Rc<FunctionInterface>,
    pub statements: Vec<Box<Statement>>,
}

pub struct Struct {
    pub id: Uuid,
    pub name: String,
}

pub struct Parameter {
    pub external_key: ParameterKey,
    pub variable: Rc<Variable>
}

#[derive(Clone, PartialEq, Eq, Hash)]
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

#[derive(Clone, PartialEq, Eq)]
pub enum Type {
    MetaType(Box<Type>),
    Primitive(primitives::Type),
    NDArray(Box<Type>),
    Struct(Rc<Struct>),
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

pub struct PassedArgumentType<'a> {
    pub key: ParameterKey,
    pub value: &'a Option<Box<Type>>,
}

// Impl

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Variable {}

impl Hash for Variable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for FunctionInterface {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for FunctionInterface {}

impl Hash for FunctionInterface {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Struct {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Struct {}

impl Hash for Struct {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Debug for PassedArgumentType<'_> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{:?}: {:?}", &self.key, &self.value)
    }
}

impl Debug for ParameterKey {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        use ParameterKey::*;
        match self {
            Name(n) => write!(fmt, "{}", n),
            Int(n) => write!(fmt, "{}", n),
        }
    }
}

impl Debug for Type {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        use Type::*;
        match self {
            Primitive(p) => write!(fmt, "{}", p.identifier_string()),
            NDArray(atom) => write!(fmt, "{:?}", atom),
            Function(f) => write!(fmt, "(?) -> ({:?})", f.return_type),
            Generic(g) => write!(fmt, "{}", g.name),
            MetaType(t) => write!(fmt, "Type[{:?}]", t),
            Struct(s) => write!(fmt, "{:?}", s.name),
        }
    }
}

impl Type {
    pub fn collect_generics<'a>(&'a self, set: &mut HashSet<&'a Rc<Generic>>) {
        match self {
            Type::MetaType(t) => t.collect_generics(set),
            Type::Primitive(_) => {}
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
            Type::Struct(s) => {}
        }
    }

    pub fn make_any() -> Box<Type> {
        Box::new(Type::Generic(Rc::new(Generic {
            id: Uuid::new_v4(),
            name: String::from("Any")
        })))
    }

    pub fn satisfies(&self, other: &Type) -> bool {
        if self == other {
            return true;
        }

        // TODO This is obviously wrong, but needed for now until generics are implemented.
        match other {
            Type::Generic(_) => true,
            _ => false
        }
    }

    pub fn arguments_satisfy_function(arguments: &Vec<PassedArgumentType>, function: &FunctionInterface) -> bool {
        if arguments.len() != function.parameters.len() {
            return false;
        }

        for (argument, parameter) in zip(arguments, &function.parameters) {
            if argument.key != parameter.external_key {
                return false;
            }

            guard!(let Some(argument_value) = argument.value else {
                return false;
            });

            if !argument_value.satisfies(&parameter.variable.type_declaration) {
                return false;
            }
        }

        return true;
    }
}

impl PassedArgument {
    pub fn to_argument_type(&self) -> PassedArgumentType {
        PassedArgumentType {
            key: self.key.clone(),
            value: &self.value.result_type
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
