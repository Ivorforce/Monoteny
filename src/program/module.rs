use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use uuid::Uuid;
use crate::program::functions::{FunctionHead, FunctionInterface, FunctionPointer, FunctionType};
use crate::program::global::{BuiltinFunctionHint, FunctionImplementation};
use crate::program::traits::{Trait, TraitGraph};
use crate::program::types::{Pattern, TypeProto, TypeUnit};

pub struct Module {
    pub id: Uuid,
    pub name: String,

    /// For each trait, its metatype getter function.
    pub trait_by_getter: HashMap<Rc<FunctionHead>, Rc<Trait>>,

    pub patterns: HashSet<Rc<Pattern>>,
    pub trait_conformance: Box<TraitGraph>,

    /// For referencable functions, their 'default' representation for calling them.
    pub fn_pointers: HashMap<Rc<FunctionHead>, Rc<FunctionPointer>>,
    /// For referencable functions, a provider function to get it as a function.
    /// FIXME We don't really need to keep track of these. This could be a builtin hint.
    ///  This requires us to provide a FunctionImplementation for call_as_function on every function
    ///  object. Next commit.
    pub fn_getters: HashMap<Rc<FunctionHead>, Rc<FunctionHead>>,
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
            trait_by_getter: Default::default(),
            patterns: Default::default(),
            trait_conformance: Box::new(TraitGraph::new()),
            fn_pointers: Default::default(),
            fn_getters: Default::default(),
            fn_implementations: Default::default(),
            fn_builtin_hints: Default::default(),
            main_functions: vec![],
            transpile_functions: vec![],
        }
    }

    pub fn add_trait(&mut self, trait_: &Rc<Trait>) -> Rc<FunctionPointer> {
        let meta_type = TypeProto::meta(TypeProto::unit(TypeUnit::Struct(Rc::clone(trait_))));
        let getter = FunctionPointer::new_global_implicit(trait_.name.as_str(), FunctionInterface::new_provider(&meta_type, vec![]));

        self.trait_by_getter.insert(
            Rc::clone(&getter.target),
            Rc::clone(trait_),
        );
        self.add_function(Rc::clone(&getter));
        getter
    }

    pub fn add_function(&mut self, function: Rc<FunctionPointer>) {
        let getter = FunctionHead::new(
            FunctionInterface::new_provider(&TypeProto::unit(TypeUnit::Function(Rc::clone(&function.target))), vec![]),
            FunctionType::Static
        );
        self.fn_getters.insert(Rc::clone(&function.target), getter);

        self.fn_pointers.insert(
            Rc::clone(&function.target),
            function,
        );
    }
}
