use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use uuid::Uuid;
use crate::linker::precedence::PrecedenceGroup;
use crate::program::functions::FunctionPointer;
use crate::program::traits::{Trait, TraitConformanceDeclaration};
use crate::program::types::Pattern;

pub struct Module {
    pub id: Uuid,
    pub name: String,

    pub traits: HashSet<Rc<Trait>>,
    pub functions: HashSet<Rc<FunctionPointer>>,
    pub patterns: HashSet<Rc<Pattern>>,
    pub trait_conformance_declarations: HashSet<Rc<TraitConformanceDeclaration>>
}

impl Module {
    pub fn new(name: String) -> Module {
        Module {
            id: Default::default(),
            name,
            traits: Default::default(),
            functions: Default::default(),
            patterns: Default::default(),
            trait_conformance_declarations: Default::default(),
        }
    }
}
