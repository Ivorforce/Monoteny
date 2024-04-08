use uuid::Uuid;
use std::hash::Hash;
use std::rc::Rc;
use std::fmt::{Debug, Formatter};
use std::collections::{HashMap, HashSet};
use itertools::Itertools;
use crate::program::traits::{Trait};
use crate::program::generics::GenericAlias;
use crate::util::fmt::write_comma_separated_list_debug;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TypeProto {
    pub unit: TypeUnit,
    pub arguments: Vec<Rc<TypeProto>>
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum TypeUnit {
    /// Used because the expression_forest wants to bind a return type for an expression.
    ///  If none is bound, that would rather indicate an error.
    ///  If one is bound, and it's void, that means we KNOW it has return type void.
    /// Having it doesn't hurt anyway; an implementation might actually pass void objects around
    ///  to simplify logic.
    Void,
    /// some type that isn't bound yet. This is fully unique and should not be created statically or imported.
    Generic(GenericAlias),
    /// Bound to an instance of a trait. The arguments are the generic bindings.
    Struct(Rc<Trait>),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Generic {
    pub id: Uuid,
    pub name: String,
}

impl Debug for TypeProto {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{:?}", self.unit)?;
        if !self.arguments.is_empty() {
            write!(fmt, "<")?;
            write_comma_separated_list_debug(fmt, &self.arguments)?;
            write!(fmt, ">")?;
        }
        Ok(())
    }
}

impl Debug for TypeUnit {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeUnit::Struct(s) => write!(fmt, "{:?}", s),
            TypeUnit::Generic(g) => write!(fmt, "#({})", g),
            TypeUnit::Void => write!(fmt, "Void"),
        }
    }
}

impl TypeProto {
    pub fn void() -> Rc<TypeProto> {
        TypeProto::unit(TypeUnit::Void)
    }

    pub fn unit(unit: TypeUnit) -> Rc<TypeProto> {
        Rc::new(TypeProto { unit, arguments: vec![] })
    }

    pub fn one_arg(trait_: &Rc<Trait>, subtype: Rc<TypeProto>) -> Rc<TypeProto> {
        Rc::new(TypeProto {
            unit: TypeUnit::Struct(Rc::clone(trait_)),
            arguments: vec![subtype]
        })
    }

    pub fn unit_struct(trait_: &Rc<Trait>) -> Rc<TypeProto> {
        TypeProto::unit(TypeUnit::Struct(Rc::clone(trait_)))
    }

    pub fn replacing_generics(self: &Rc<TypeProto>, map: &HashMap<Uuid, Rc<TypeProto>>) -> Rc<TypeProto> {
        match &self.unit {
            TypeUnit::Generic(id) => map.get(id)
                .cloned()
                .unwrap_or_else(|| self.clone()),
            _ => Rc::new(TypeProto {
                unit: self.unit.clone(),
                arguments: self.arguments.iter().map(|x| x.replacing_generics(map)).collect()
            }),
        }
    }

    pub fn replacing_structs(self: &Rc<TypeProto>, map: &HashMap<Rc<Trait>, Rc<TypeProto>>) -> Rc<TypeProto> {
        match &self.unit {
            TypeUnit::Struct(struct_) => map.get(struct_)
                .cloned()
                .unwrap_or_else(|| self.clone()),
            _ => Rc::new(TypeProto {
                unit: self.unit.clone(),
                arguments: self.arguments.iter().map(|x| x.replacing_structs(map)).collect()
            }),
        }
    }

    pub fn collect_generics<'a, C>(collection: C) -> HashSet<Uuid> where C: Iterator<Item=&'a Rc<TypeProto>> {
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

    pub fn contains_generics<'a, C>(collection: C) -> bool where C: Iterator<Item=&'a Rc<TypeProto>> {
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
