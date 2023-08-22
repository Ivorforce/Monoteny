use uuid::Uuid;
use std::hash::Hash;
use std::rc::Rc;
use std::fmt::{Debug, Formatter, Pointer};
use std::collections::HashMap;
use std::ops::BitXor;
use crate::program::traits::{Trait};
use crate::linker::precedence::PrecedenceGroup;
use crate::program::functions::{FunctionPointer, ParameterKey};
use crate::program::generics::GenericAlias;
use crate::util::fmt::write_comma_separated_list;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TypeProto {
    pub unit: TypeUnit,
    pub arguments: Vec<Box<TypeProto>>
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum TypeUnit {
    // Used because the expression_forest wants to bind a return type for an expression.
    //  If none is bound, that would rather indicate an error.
    //  If one is bound, and it's void, that means we KNOW it has return type void.
    // Having it doesn't hurt anyway; an implementation might actually pass void objects around
    //  to simplify logic.
    Void,  // Not a type
    MetaType,  // Type of a type
    Any(Uuid),  // some unknown type - may be described by requirements
    Generic(GenericAlias),  // some type that isn't bound yet
    Monad,  // Bound to a monad with arguments [unit, dimensions...]
    Struct(Rc<Trait>),  // Bound to an instance of some trait (non-abstract)
    Function(Rc<FunctionPointer>),  // Bound to a function / reference to a function.
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
            Struct(s) => write!(fmt, "{}", s.name),
            Monad => write!(fmt, "Monad"),
            Generic(g) => write!(fmt, "Generic<{}>", g),
            Any(g) => write!(fmt, "Any<{}>", g),
            MetaType => write!(fmt, "MetaType"),
            Void => write!(fmt, "Void"),
            Function(f) => write!(fmt, "{:?}", f),
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

    pub fn simple_struct(trait_: &Rc<Trait>) -> Box<TypeProto> {
        TypeProto::unit(TypeUnit::Struct(Rc::clone(trait_)))
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
