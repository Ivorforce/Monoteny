use uuid::Uuid;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::fmt::{Debug, Formatter};
use std::collections::{HashMap, HashSet};
use std::ops::BitXor;
use guard::guard;
use crate::program::traits::{Trait, TraitConformanceRequirement};
use crate::linker::precedence::{OperatorAssociativity, PrecedenceGroup};
use crate::program::functions::{FunctionOverload, FunctionPointer, HumanFunctionInterface, ParameterKey};
use crate::program::generics::{GenericAlias, TypeForest};

use crate::program::primitives;
use crate::program::structs::Struct;
use crate::util::fmt::write_comma_separated_list;

#[derive(Clone, PartialEq, Eq)]
pub struct TypeProto {
    pub unit: TypeUnit,
    pub arguments: Vec<Box<TypeProto>>
}

#[derive(Clone, PartialEq, Eq)]
pub enum TypeUnit {
    Void,  // Not a type
    MetaType,  // Type of a type
    Any(Uuid),  // Bound to some unknown type
    Generic(GenericAlias),  // Bound to a type within a GenericMapping
    Monad,
    Primitive(primitives::Type),
    Struct(Rc<Struct>),
    AnonymousStruct(Vec<ParameterKey>),
    Trait(Rc<Trait>),
    FunctionOverload(Rc<FunctionOverload>),
    PrecedenceGroup(Rc<PrecedenceGroup>),
    Keyword(String),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Generic {
    pub id: Uuid,
    pub name: String,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Pattern {
    pub id: Uuid,
    pub alias: String,
    pub precedence_group: Rc<PrecedenceGroup>,

    pub parts: Vec<Box<PatternPart>>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum PatternPart {
    Parameter { key: ParameterKey, internal_name: String },
    Keyword(String),
}

impl Debug for TypeProto {
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
            FunctionOverload(f) => write!(fmt, "FunctionOverload"),
            Generic(g) => write!(fmt, "Generic<{}>", g),
            Any(g) => write!(fmt, "Any<{}>", g),
            MetaType => write!(fmt, "MetaType"),
            PrecedenceGroup(p) => write!(fmt, "{:?}", p.name),
            Void => write!(fmt, "Void"),
            AnonymousStruct(names) => {
                write!(fmt, "(")?;
                write_comma_separated_list(fmt, names)?;
                write!(fmt, ")")
            }
            Keyword(s) => write!(fmt, "{}", s),
        }
    }
}

impl TypeProto {
    pub fn make_any() -> Box<TypeProto> {
        TypeProto::unit(TypeUnit::Any(Uuid::new_v4()))
    }

    pub fn void() -> Box<TypeProto> {
        TypeProto::unit(TypeUnit::Void)
    }

    pub fn unit(unit: TypeUnit) -> Box<TypeProto> {
        Box::new(TypeProto { unit, arguments: vec![] })
    }

    pub fn meta(subtype: Box<TypeProto>) -> Box<TypeProto> {
        Box::new(TypeProto {
            unit: TypeUnit::MetaType,
            arguments: vec![subtype]
        })
    }

    pub fn monad(unit: Box<TypeProto>) -> Box<TypeProto> {
        Box::new(TypeProto { unit: TypeUnit::Monad, arguments: vec![unit] })
    }

    fn bitxor(lhs: &Uuid, rhs: &Uuid) -> Uuid {
        Uuid::from_u128(lhs.as_u128() ^ rhs.as_u128())
    }

    pub fn with_any_as_generic(&self, seed: &Uuid) -> Box<TypeProto> {
        Box::new(TypeProto {
            unit: match &self.unit {
                TypeUnit::Any(id) => TypeUnit::Generic(TypeProto::bitxor(seed, id)),
                _ => self.unit.clone(),
            },
            arguments: self.arguments.iter().map(|x| x.with_any_as_generic(seed)).collect()
        })
    }

    pub fn replacing_any(&self, map: &HashMap<Uuid, Box<TypeProto>>) -> Box<TypeProto> {
        match &self.unit {
            TypeUnit::Any(id) => map.get(id)
                .map(|x| x.clone())
                .unwrap_or_else(|| Box::new(self.clone())),
            _ => Box::new(TypeProto {
                unit: self.unit.clone(),
                arguments: self.arguments.iter().map(|x| x.replacing_any(map)).collect()
            }),
        }
    }
}

impl TypeUnit {
    pub fn is_void(&self) -> bool {
        match self {
            TypeUnit::Void => true,
            _ => false
        }
    }
}
