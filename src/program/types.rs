use uuid::Uuid;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::fmt::{Debug, Formatter};
use std::collections::{HashMap, HashSet};
use std::ops::BitXor;
use guard::guard;
use crate::program::traits::{Trait, TraitConformanceRequirement};
use crate::parser::associativity::{OperatorAssociativity, PrecedenceGroup};
use crate::program::functions::{FunctionPointer, HumanFunctionInterface};
use crate::program::generics::GenericMapping;

use crate::program::primitives;
use crate::util::fmt::write_comma_separated_list;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Mutability {
    Immutable,
    Mutable,
}

pub struct Struct {
    pub id: Uuid,
    pub name: String,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ParameterKey {
    Positional,
    Name(String),
}

#[derive(Clone)]
pub struct Variable {
    pub id: Uuid,
    pub type_declaration: Box<Type>,
    pub mutability: Mutability
}

#[derive(Clone, PartialEq, Eq)]
pub struct Type {
    pub unit: TypeUnit,
    pub arguments: Vec<Box<Type>>
}

#[derive(Clone, PartialEq, Eq)]
pub enum TypeUnit {
    MetaType,  // Type of a type
    Any(Uuid),  // Bound to some unknown type
    Generic(Uuid),  // Bound to a type within a GenericMapping
    Monad,
    Primitive(primitives::Type),
    Struct(Rc<Struct>),
    Trait(Rc<Trait>),
    Function(Rc<FunctionPointer>),
    PrecedenceGroup(Rc<PrecedenceGroup>),
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
            Name(s) => write!(fmt, "{}", s),
            Positional => write!(fmt, "_"),
        }
    }
}

impl Debug for Type {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{:?}", self.unit)?;
        if !self.arguments.is_empty() {
            write!(fmt, "<")?;
            write_comma_separated_list(fmt, &self.arguments)?;
            write!(fmt, ">")?;
        }
        Ok(())
    }
}

impl Debug for TypeUnit {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        use TypeUnit::*;
        match self {
            Primitive(p) => write!(fmt, "{}", p.identifier_string()),
            Struct(s) => write!(fmt, "{:?}", s.name),
            Trait(t) => write!(fmt, "{:?}", t.name),
            Monad => write!(fmt, "Monad"),
            Function(f) => write!(fmt, "(?) -> ({:?})", f.machine_interface.return_type),
            Generic(g) => write!(fmt, "Generic<{}>", g),
            Any(g) => write!(fmt, "Any<{}>", g),
            MetaType => write!(fmt, "MetaType"),
            PrecedenceGroup(p) => write!(fmt, "{:?}", p.name),
        }
    }
}

impl Type {
    pub fn make_any() -> Box<Type> {
        Type::unit(TypeUnit::Any(Uuid::new_v4()))
    }

    pub fn unit(unit: TypeUnit) -> Box<Type> {
        Box::new(Type { unit, arguments: vec![] })
    }

    pub fn meta(subtype: Box<Type>) -> Box<Type> {
        Box::new(Type {
            unit: TypeUnit::MetaType,
            arguments: vec![subtype]
        })
    }

    fn bitxor(lhs: &Uuid, rhs: &Uuid) -> Uuid {
        Uuid::from_u128(lhs.as_u128() ^ rhs.as_u128())
    }

    pub fn with_any_as_generic(&self, seed: &Uuid) -> Box<Type> {
        Box::new(Type {
            unit: match &self.unit {
                TypeUnit::Any(id) => TypeUnit::Generic(Type::bitxor(seed, id)),
                _ => self.unit.clone(),
            },
            arguments: self.arguments.iter().map(|x| x.with_any_as_generic(seed)).collect()
        })
    }

    pub fn replacing_any(&self, map: &HashMap<Uuid, Box<Type>>) -> Box<Type> {
        match &self.unit {
            TypeUnit::Any(id) => map.get(id)
                .map(|x| x.clone())
                .unwrap_or_else(|| Box::new(self.clone())),
            _ => Box::new(Type {
                unit: self.unit.clone(),
                arguments: self.arguments.iter().map(|x| x.replacing_any(map)).collect()
            }),
        }
    }
}

impl Variable {
    pub fn make_immutable(type_declaration: Box<Type>) -> Rc<Variable> {
        Rc::new(Variable {
            id: Uuid::new_v4(),
            type_declaration,
            mutability: Mutability::Immutable
        })
    }
}

impl ParameterKey {
    pub fn from_string(s: String) -> ParameterKey {
        if s == "_" { ParameterKey::Positional } else { ParameterKey::Name(s) }
    }
}
