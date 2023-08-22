use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use uuid::Uuid;
use crate::program::allocation::ObjectReference;
use crate::program::functions::FunctionPointer;
use crate::program::traits::{Trait, TraitGraph};
use crate::program::types::{Pattern, TypeProto, TypeUnit};

pub struct Module {
    pub id: Uuid,
    pub name: String,

    pub traits: HashMap<Rc<Trait>, Rc<ObjectReference>>,
    pub functions: HashMap<Rc<FunctionPointer>, Rc<ObjectReference>>,
    pub patterns: HashSet<Rc<Pattern>>,
    pub trait_conformance: Box<TraitGraph>
}

impl Module {
    pub fn new(name: String) -> Module {
        Module {
            id: Default::default(),
            name,
            traits: Default::default(),
            functions: Default::default(),
            patterns: Default::default(),
            trait_conformance: Box::new(TraitGraph::new()),
        }
    }

    pub fn add_trait(&mut self, trait_: &Rc<Trait>) {
        self.traits.insert(
            Rc::clone(trait_),
            ObjectReference::new_immutable(TypeProto::meta(TypeProto::unit(TypeUnit::Struct(Rc::clone(trait_)))))
        );
    }

    pub fn add_function(&mut self, function: &Rc<FunctionPointer>) {
        self.functions.insert(
            Rc::clone(function),
            ObjectReference::new_immutable(TypeProto::unit(TypeUnit::Function(Rc::clone(function))))
        );
    }
}
