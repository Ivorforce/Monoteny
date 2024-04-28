use std::collections::HashSet;
use std::rc::Rc;

use itertools::Itertools;
use uuid::Uuid;
use crate::parser::grammar::{Pattern, PrecedenceGroup};

use crate::program::functions::FunctionHead;
use crate::program::traits::TraitGraph;
use crate::source::Source;

pub type ModuleName = Vec<String>;

pub fn module_name(name: &str) -> ModuleName {
    name.split(".").map(ToString::to_string).collect_vec()
}

pub struct Module {
    pub id: Uuid,
    pub name: ModuleName,

    /// For each trait, its metatype getter function.
    pub included_modules: Vec<Vec<String>>,

    pub precedence_order: Option<Vec<Rc<PrecedenceGroup>>>,
    pub patterns: HashSet<Rc<Pattern<Rc<FunctionHead>>>>,
    pub trait_conformance: Box<TraitGraph>,

    /// Functions that are directly referencible.
    /// Usually, these are just getters for traits, function objects etc.
    pub exposed_functions: HashSet<Rc<FunctionHead>>,

    /// These come from decorators.
    /// Collecting all decorated functions allows us to fail late - the rest of the code is still
    ///  valid even if multiple main! functions are declared! We just cannot run them as 'main'.
    pub main_functions: Vec<Rc<FunctionHead>>,
    pub transpile_functions: Vec<Rc<FunctionHead>>,
}

impl Module {
    pub fn new(name: ModuleName) -> Module {
        Module {
            id: Default::default(),
            name,
            included_modules: vec![],
            precedence_order: None,
            patterns: Default::default(),
            trait_conformance: Box::new(TraitGraph::new()),
            exposed_functions: Default::default(),
            main_functions: vec![],
            transpile_functions: vec![],
        }
    }
}

impl Module {
    pub fn explicit_functions<'a>(&'a self, source: &'a Source) -> Vec<&'a Rc<FunctionHead>> {
        self.exposed_functions.iter().collect_vec()
    }
}
