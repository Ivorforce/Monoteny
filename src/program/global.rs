use std::rc::Rc;
use uuid::Uuid;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use crate::program::computation_tree::{ExpressionForest, Statement};
use crate::program::functions::{Function, FunctionInterface, FunctionPointer};
use crate::program::traits::{Trait, TraitConformanceDeclaration, TraitRequirement};
use crate::program::allocation::{ObjectReference, Reference};
use crate::program::generics::TypeForest;

pub struct FunctionImplementation {
    pub implementation_id: Uuid,
    pub pointer: Rc<FunctionPointer>,
    pub decorators: Vec<String>,

    pub conformance_delegations: HashMap<Rc<TraitRequirement>, Rc<TraitConformanceDeclaration>>,

    pub statements: Vec<Box<Statement>>,
    pub expression_forest: Box<ExpressionForest>,
    pub type_forest: Box<TypeForest>,
    pub variable_names: HashMap<Rc<ObjectReference>, String>,
}

impl PartialEq for FunctionImplementation {
    fn eq(&self, other: &Self) -> bool {
        self.implementation_id == other.implementation_id
    }
}

impl Eq for FunctionImplementation {}

impl Hash for FunctionImplementation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.implementation_id.hash(state);
    }
}
