use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;
use uuid::Uuid;
use crate::{linker, parser};
use crate::linker::LinkError;
use crate::program::allocation::ObjectReference;
use crate::program::builtins::Builtins;
use crate::program::functions::{FunctionHead, FunctionPointer};
use crate::program::global::{BuiltinFunctionHint, FunctionImplementation};
use crate::program::traits::{Trait, TraitGraph};
use crate::program::types::{Pattern, TypeProto, TypeUnit};

pub struct Module {
    pub id: Uuid,
    pub name: String,

    pub traits: HashMap<Rc<Trait>, Rc<ObjectReference>>,
    pub patterns: HashSet<Rc<Pattern>>,
    pub trait_conformance: Box<TraitGraph>,

    /// For each function, a usable reference to it as an object.
    pub fn_references: HashMap<Rc<FunctionHead>, Rc<ObjectReference>>,
    /// For each function, its 'default' representation for syntax.
    pub fn_pointers: HashMap<Rc<FunctionHead>, Rc<FunctionPointer>>,
    /// For relevant functions, their implementation.
    pub fn_implementations: HashMap<Rc<FunctionHead>, Box<FunctionImplementation>>,
    /// For relevant functions, a hint what type of builtin it is.
    pub fn_builtin_hints: HashMap<Rc<FunctionHead>, BuiltinFunctionHint>,

    /// These come from decorators.
    /// Collecting all decorated functions allows us to fail late - the rest of the code is still
    ///  valid even if multiple @main functions are declared! We just cannot run them as 'main'.
    pub main_functions: Vec<Rc<FunctionHead>>,
    pub transpile_functions: Vec<Rc<FunctionHead>>,
}

impl Module {
    pub fn new(name: String) -> Module {
        Module {
            id: Default::default(),
            name,
            traits: Default::default(),
            fn_references: Default::default(),
            patterns: Default::default(),
            trait_conformance: Box::new(TraitGraph::new()),
            fn_implementations: Default::default(),
            fn_builtin_hints: Default::default(),
            fn_pointers: Default::default(),
            main_functions: vec![],
            transpile_functions: vec![],
        }
    }

    pub fn add_trait(&mut self, trait_: &Rc<Trait>) -> Rc<ObjectReference> {
        let reference = ObjectReference::new_immutable(TypeProto::meta(TypeProto::unit(TypeUnit::Struct(Rc::clone(trait_)))));
        self.traits.insert(
            Rc::clone(trait_),
            Rc::clone(&reference)
        );
        reference
    }

    pub fn add_function(&mut self, function: &Rc<FunctionPointer>) {
        self.fn_references.insert(
            Rc::clone(&function.target),
            ObjectReference::new_immutable(TypeProto::unit(TypeUnit::Function(Rc::clone(&function.target))))
        );
        self.fn_pointers.insert(
            Rc::clone(&function.target),
            Rc::clone(&function),
        );
    }
}
