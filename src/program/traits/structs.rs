use std::rc::Rc;
use std::collections::HashMap;
use crate::program::allocation::ObjectReference;
use crate::program::functions::FunctionHead;
use crate::program::traits::Trait;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StructInfo {
    pub trait_: Rc<Trait>,

    pub clone: Rc<FunctionHead>,
    pub constructor: Rc<FunctionHead>,
    pub fields: Vec<Rc<ObjectReference>>,

    pub field_names: HashMap<Rc<ObjectReference>, String>,
    pub field_getters: HashMap<Rc<ObjectReference>, Rc<FunctionHead>>,
    pub field_setters: HashMap<Rc<ObjectReference>, Rc<FunctionHead>>,
}
