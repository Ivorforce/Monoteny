use uuid::Uuid;
use std::hash::Hash;
use std::rc::Rc;
use std::fmt::{Debug, Formatter, Pointer};
use std::collections::{HashMap, HashSet};
use std::ops::BitXor;
use itertools::Itertools;
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
    Struct(Rc<Trait>),  // Bound to a plain instance of some trait
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

    pub fn bitxor(lhs: &Uuid, rhs: &Uuid) -> Uuid {
        Uuid::from_u128(lhs.as_u128() ^ rhs.as_u128())
    }

    pub fn freezing_generics_to_any(&self) -> Box<TypeProto> {
        Box::new(TypeProto {
            unit: match &self.unit {
                TypeUnit::Generic(id) => TypeUnit::Any(*id),
                _ => self.unit.clone(),
            },
            arguments: self.arguments.iter().map(|x| x.freezing_generics_to_any()).collect()
        })
    }

    pub fn unfreezing_any_to_generics(&self) -> Box<TypeProto> {
        Box::new(TypeProto {
            unit: match &self.unit {
                TypeUnit::Any(id) => TypeUnit::Generic(*id),
                _ => self.unit.clone(),
            },
            arguments: self.arguments.iter().map(|x| x.unfreezing_any_to_generics()).collect()
        })
    }

    pub fn seeding_generics(&self, seed: &Uuid) -> Box<TypeProto> {
        Box::new(TypeProto {
            unit: match &self.unit {
                TypeUnit::Generic(id) => TypeUnit::Generic(TypeProto::bitxor(seed, id)),
                _ => self.unit.clone(),
            },
            arguments: self.arguments.iter().map(|x| x.seeding_generics(seed)).collect()
        })
    }

    pub fn replacing_generics(&self, map: &HashMap<Uuid, Box<TypeProto>>) -> Box<TypeProto> {
        match &self.unit {
            TypeUnit::Generic(id) => map.get(id)
                .map(|x| x.clone())
                .unwrap_or_else(|| Box::new(self.clone())),
            _ => Box::new(TypeProto {
                unit: self.unit.clone(),
                arguments: self.arguments.iter().map(|x| x.replacing_generics(map)).collect()
            }),
        }
    }

    pub fn replacing_anys(&self, map: &HashMap<Uuid, Box<TypeProto>>) -> Box<TypeProto> {
        match &self.unit {
            TypeUnit::Any(id) => map.get(id)
                .map(|x| x.clone())
                .unwrap_or_else(|| Box::new(self.clone())),
            _ => Box::new(TypeProto {
                unit: self.unit.clone(),
                arguments: self.arguments.iter().map(|x| x.replacing_generics(map)).collect()
            }),
        }
    }

    pub fn collect_generics<'a, C>(collection: C) -> HashSet<Uuid> where C: Iterator<Item=&'a Box<TypeProto>> {
        let mut anys = HashSet::new();
        let mut todo = collection.collect_vec();

        while let Some(next) = todo.pop() {
            match &next.unit {
                TypeUnit::Generic(id) => { anys.insert(*id); },
                _ => {}
            };
            todo.extend(&next.arguments);
        }

        anys
    }

    pub fn contains_generics<'a, C>(collection: C) -> bool where C: Iterator<Item=&'a Box<TypeProto>> {
        let mut todo = collection.collect_vec();

        while let Some(next) = todo.pop() {
            match &next.unit {
                TypeUnit::Generic(_) => { return true },
                _ => {}
            };
            todo.extend(&next.arguments);
        }

        return false
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
