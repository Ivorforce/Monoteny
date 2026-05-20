use crate::program::allocation::ObjectReference;
use crate::program::functions::FunctionHead;
use crate::program::traits::Nominal;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StructInfo {
    pub nominal: Rc<Nominal>,

    pub clone: Rc<FunctionHead>,
    pub constructor: Rc<FunctionHead>,
    pub fields: Vec<Rc<ObjectReference>>,

    pub field_names: HashMap<Rc<ObjectReference>, String>,
    pub field_getters: HashMap<Rc<ObjectReference>, Rc<FunctionHead>>,
    pub field_setters: HashMap<Rc<ObjectReference>, Rc<FunctionHead>>,
}

impl Hash for StructInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.nominal.hash(state)
    }
}
