use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;
use uuid::Uuid;
use crate::{linker, parser};
use crate::linker::LinkError;
use crate::program::allocation::ObjectReference;
use crate::program::builtins::Builtins;
use crate::program::functions::FunctionPointer;
use crate::program::global::{BuiltinFunctionHint, FunctionImplementation};
use crate::program::traits::{Trait, TraitGraph};
use crate::program::types::{Pattern, TypeProto, TypeUnit};

pub struct Module {
    pub id: Uuid,
    pub name: String,

    pub traits: HashMap<Rc<Trait>, Rc<ObjectReference>>,
    pub functions: HashMap<Rc<FunctionPointer>, Rc<ObjectReference>>,
    pub patterns: HashSet<Rc<Pattern>>,
    pub trait_conformance: Box<TraitGraph>,
    pub function_implementations: HashMap<Rc<FunctionPointer>, Rc<FunctionImplementation>>,
    pub builtin_hints: HashMap<Rc<FunctionPointer>, BuiltinFunctionHint>,
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
            function_implementations: Default::default(),
            builtin_hints: Default::default(),
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
        self.functions.insert(
            Rc::clone(function),
            ObjectReference::new_immutable(TypeProto::unit(TypeUnit::Function(Rc::clone(function))))
        );
    }
}

pub fn from_file(path: PathBuf, builtins: &Builtins) -> Result<Rc<Module>, LinkError> {
    let content = std::fs::read_to_string(&path)
        .expect("could not read library file");

    let syntax_tree = parser::parse_program(&content);

    let builtin_variable_scope = builtins.create_scope();
    let module = linker::link_file(syntax_tree, &builtin_variable_scope, &builtins)?;

    Ok(module)
}
