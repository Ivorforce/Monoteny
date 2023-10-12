use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use itertools::Itertools;
use uuid::Uuid;
use crate::linker::precedence::PrecedenceGroup;
use crate::program::function_object::{FunctionForm, FunctionRepresentation};
use crate::program::functions::{FunctionHead, FunctionInterface, FunctionType};
use crate::program::global::{BuiltinFunctionHint, FunctionImplementation};
use crate::program::traits::{Trait, TraitGraph};
use crate::program::types::{Pattern, TypeProto, TypeUnit};

pub type ModuleName = Vec<String>;

pub fn module_name(name: &str) -> ModuleName {
    name.split(".").map(ToString::to_string).collect_vec()
}

pub struct Module {
    pub id: Uuid,
    pub name: ModuleName,

    /// For each trait, its metatype getter function.
    pub included_modules: Vec<Vec<String>>,

    /// For each trait, its metatype getter function.
    pub trait_by_getter: HashMap<Rc<FunctionHead>, Rc<Trait>>,

    pub precedence_order: Option<Vec<Rc<PrecedenceGroup>>>,
    pub patterns: HashSet<Rc<Pattern>>,
    pub trait_conformance: Box<TraitGraph>,

    /// For referencable functions, their 'default' representation for calling them.
    pub fn_representations: HashMap<Rc<FunctionHead>, FunctionRepresentation>,
    /// For referencable functions, a provider function to get it as a function.
    /// FIXME We don't really need to keep track of these. This could be a core hint.
    ///  This requires us to provide a FunctionImplementation for call_as_function on every function
    ///  object. Next commit.
    pub fn_getters: HashMap<Rc<FunctionHead>, Rc<FunctionHead>>,
    /// For relevant functions, their implementation.
    pub fn_implementations: HashMap<Rc<FunctionHead>, Box<FunctionImplementation>>,
    /// For relevant functions, a hint what type of core it is.
    pub fn_builtin_hints: HashMap<Rc<FunctionHead>, BuiltinFunctionHint>,

    /// These come from decorators.
    /// Collecting all decorated functions allows us to fail late - the rest of the code is still
    ///  valid even if multiple @main functions are declared! We just cannot run them as 'main'.
    pub main_functions: Vec<Rc<FunctionHead>>,
    pub transpile_functions: Vec<Rc<FunctionHead>>,
}

impl Module {
    pub fn new(name: ModuleName) -> Module {
        Module {
            id: Default::default(),
            name,
            included_modules: vec![],
            trait_by_getter: Default::default(),
            precedence_order: None,
            patterns: Default::default(),
            trait_conformance: Box::new(TraitGraph::new()),
            fn_representations: Default::default(),
            fn_getters: Default::default(),
            fn_implementations: Default::default(),
            fn_builtin_hints: Default::default(),
            main_functions: vec![],
            transpile_functions: vec![],
        }
    }

    pub fn add_trait(&mut self, trait_: &Rc<Trait>) -> Rc<FunctionHead> {
        let meta_type = TypeProto::meta(TypeProto::unit(TypeUnit::Struct(Rc::clone(trait_))));
        let getter = FunctionHead::new_static(FunctionInterface::new_provider(&meta_type, vec![]));

        self.trait_by_getter.insert(
            Rc::clone(&getter),
            Rc::clone(trait_),
        );
        self.add_function(Rc::clone(&getter), FunctionRepresentation::new(&trait_.name, FunctionForm::GlobalImplicit));
        getter
    }

    pub fn add_function(&mut self, function: Rc<FunctionHead>, representation: FunctionRepresentation) {
        let getter = FunctionHead::new_static(
            FunctionInterface::new_provider(&TypeProto::unit(TypeUnit::Function(Rc::clone(&function))), vec![]),
        );
        self.fn_getters.insert(Rc::clone(&function), getter);

        self.fn_representations.insert(
            function,
            representation,
        );
    }
}
