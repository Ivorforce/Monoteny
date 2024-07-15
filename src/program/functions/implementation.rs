use std::collections::HashMap;
use std::rc::Rc;

use crate::program::allocation::ObjectReference;
use crate::program::expression_tree::ExpressionTree;
use crate::program::functions::FunctionHead;
use crate::program::generics::TypeForest;
use crate::program::traits::RequirementsAssumption;

#[derive(Clone)]
pub struct FunctionImplementation {
    pub head: Rc<FunctionHead>,

    pub requirements_assumption: Box<RequirementsAssumption>,

    pub expression_tree: Box<ExpressionTree>,
    pub type_forest: Box<TypeForest>,

    pub parameter_locals: Vec<Rc<ObjectReference>>,
    pub locals_names: HashMap<Rc<ObjectReference>, String>,
}
