use uuid::Uuid;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::fmt::{Debug, Formatter};
use std::collections::HashSet;
use std::iter::zip;
use guard::guard;
use crate::parser::associativity::{OperatorAssociativity, PrecedenceGroup};

use crate::program::primitives;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Mutability {
    Immutable,
    Mutable,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FunctionForm {
    Global,
    Member,
    Operator,
}

pub struct FunctionInterface {
    pub id: Uuid,
    pub name: String,
    pub alphanumeric_name: String,

    pub form: FunctionForm,
    pub parameters: Vec<Box<NamedParameter>>,
    pub generics: Vec<Rc<Generic>>,

    pub return_type: Option<Box<Type>>,
}

pub struct Struct {
    pub id: Uuid,
    pub name: String,
}

pub struct NamedParameter {
    pub external_key: ParameterKey,
    pub variable: Rc<Variable>
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ParameterKey {
    Name(String),
    Int(i32),
}

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
    Monad(Box<Type>),
    Struct(Rc<Struct>),
    Function(Rc<FunctionInterface>),
    PrecedenceGroup(Rc<PrecedenceGroup>),
    Generic(Rc<Generic>),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Generic {
    pub id: Uuid,
    pub name: String,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Pattern {
    pub id: Uuid,
    pub operator: String,
    pub alias: String,
    pub precedence_group: Rc<PrecedenceGroup>,
}

pub struct PassedArgumentType<'a> {
    pub key: ParameterKey,
    pub value: &'a Option<Box<Type>>,
}

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
            Monad(unit) => write!(fmt, "{:?}", unit),
            Function(f) => write!(fmt, "(?) -> ({:?})", f.return_type),
            Generic(g) => write!(fmt, "{}", g.name),
            MetaType(t) => write!(fmt, "Type[{:?}]", t),
            Struct(s) => write!(fmt, "{:?}", s.name),
            PrecedenceGroup(p) => write!(fmt, "{:?}", p.name),
        }
    }
}

impl Type {
    pub fn collect_generics<'a>(&'a self, set: &mut HashSet<&'a Rc<Generic>>) {
        match self {
            Type::MetaType(t) => t.collect_generics(set),
            Type::Primitive(_) => {}
            Type::Monad(unit) => {
                unit.collect_generics(set);
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
            Type::PrecedenceGroup(_) => {}
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

impl Debug for PassedArgumentType<'_> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{:?}: {:?}", &self.key, &self.value)
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
