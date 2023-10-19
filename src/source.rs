use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;
use crate::program::function_object::FunctionRepresentation;
use crate::program::functions::FunctionHead;
use crate::program::global::{FunctionLogicDescriptor, FunctionImplementation};
use crate::program::module::{Module, ModuleName};
use crate::program::traits::Trait;

pub struct Source {
    pub module_by_name: HashMap<ModuleName, Box<Module>>,

    // Cache of aggregated module_by_name fields for quick reference.

    /// For every getter, which trait it provides.
    pub trait_references: HashMap<Rc<FunctionHead>, Rc<Trait>>,
    /// For referencible functions, the trait for it as an object.
    pub function_traits: HashMap<Rc<Trait>, Rc<FunctionHead>>,

    /// For each function_id, its head.
    pub fn_heads: HashMap<Uuid, Rc<FunctionHead>>,
    /// For referencible functions, a way to load it. The getter itself does not get a getter.
    pub fn_getters: HashMap<Rc<FunctionHead>, Rc<FunctionHead>>,
    /// For referencible functions, its 'default' representation for syntax.
    pub fn_representations: HashMap<Rc<FunctionHead>, FunctionRepresentation>,
    /// For relevant functions, their implementation.
    pub fn_implementations: HashMap<Rc<FunctionHead>, Box<FunctionImplementation>>,
    /// For relevant functions, a hint what type of core it is.
    pub fn_logic_descriptors: HashMap<Rc<FunctionHead>, FunctionLogicDescriptor>,
}

impl Source {
    pub fn new() -> Source {
        Source {
            module_by_name: Default::default(),
            trait_references: Default::default(),
            function_traits: Default::default(),
            fn_heads: Default::default(),
            fn_getters: Default::default(),
            fn_representations: Default::default(),
            fn_implementations: Default::default(),
            fn_logic_descriptors: Default::default(),
        }
    }
}
