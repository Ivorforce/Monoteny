use std::collections::HashMap;
use std::rc::Rc;

use uuid::Uuid;

use crate::program::functions::{FunctionHead, FunctionLogic};
use crate::program::module::{Module, ModuleName};
use crate::program::traits::Nominal;

pub struct Source {
    pub module_by_name: HashMap<ModuleName, Box<Module>>,

    // Cache of aggregated module_by_name fields for quick reference.

    /// For every getter, which nominal it provides.
    pub nominal_heads: HashMap<Uuid, Rc<Nominal>>,
    /// For referencible functions, the nominal for it as an object.
    /// For every getter, which nominal it provides.
    pub nominal_references: HashMap<Rc<FunctionHead>, Rc<Nominal>>,
    /// For referencible functions, the nominal for it as an object.
    pub function_nominals: HashMap<Rc<Nominal>, Rc<FunctionHead>>,

    /// For each function_id, its head.
    pub fn_heads: HashMap<Uuid, Rc<FunctionHead>>,
    /// For referencible functions, a way to load it. The getter itself does not get a getter.
    pub fn_getters: HashMap<Rc<FunctionHead>, Rc<FunctionHead>>,
    
    /// For all functions, their logic.
    pub fn_logic: HashMap<Rc<FunctionHead>, FunctionLogic>,
}

impl Source {
    pub fn new() -> Source {
        Source {
            module_by_name: Default::default(),
            nominal_heads: Default::default(),
            nominal_references: Default::default(),
            function_nominals: Default::default(),
            fn_heads: Default::default(),
            fn_getters: Default::default(),
            fn_logic: Default::default(),
        }
    }
}
