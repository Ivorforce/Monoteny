use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;
use crate::program::allocation::ObjectReference;
use crate::program::function_object::FunctionRepresentation;
use crate::program::functions::FunctionHead;
use crate::program::global::FunctionLogic;
use crate::program::module::{Module, ModuleName};
use crate::program::traits::Trait;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StructInfo {
    pub trait_: Rc<Trait>,

    pub constructor: Rc<FunctionHead>,
    pub fields: Vec<Rc<ObjectReference>>,

    pub field_names: HashMap<Rc<ObjectReference>, String>,
    pub field_getters: HashMap<Rc<ObjectReference>, Rc<FunctionHead>>,
    pub field_setters: HashMap<Rc<ObjectReference>, Rc<FunctionHead>>,
}

pub struct Source {
    pub module_by_name: HashMap<ModuleName, Box<Module>>,

    // Cache of aggregated module_by_name fields for quick reference.

    /// For every getter, which trait it provides.
    pub trait_references: HashMap<Rc<FunctionHead>, Rc<Trait>>,
    /// For referencible functions, the trait for it as an object.
    pub function_traits: HashMap<Rc<Trait>, Rc<FunctionHead>>,
    /// For instantiatable traits, their struct info
    pub struct_by_trait: HashMap<Rc<Trait>, Rc<StructInfo>>,

    /// For each function_id, its head.
    pub fn_heads: HashMap<Uuid, Rc<FunctionHead>>,
    /// For referencible functions, a way to load it. The getter itself does not get a getter.
    pub fn_getters: HashMap<Rc<FunctionHead>, Rc<FunctionHead>>,
    
    /// For all functions, the 'default' representation for syntax.
    pub fn_representations: HashMap<Rc<FunctionHead>, FunctionRepresentation>,
    /// For all functions, their logic.
    pub fn_logic: HashMap<Rc<FunctionHead>, FunctionLogic>,
}

impl Source {
    pub fn new() -> Source {
        Source {
            module_by_name: Default::default(),
            trait_references: Default::default(),
            function_traits: Default::default(),
            struct_by_trait: Default::default(),
            fn_heads: Default::default(),
            fn_getters: Default::default(),
            fn_representations: Default::default(),
            fn_logic: Default::default(),
        }
    }
}
