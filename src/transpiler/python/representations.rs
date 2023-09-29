use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use uuid::Uuid;
use crate::program::functions::FunctionHead;
use crate::program::global::FunctionImplementation;
use crate::program::types::TypeProto;

#[derive(Clone)]
pub struct Representations {
    pub builtin_functions: HashSet<Rc<FunctionHead>>,
    pub function_representations: HashMap<Rc<FunctionHead>, FunctionRepresentation>,
    pub type_ids: HashMap<Box<TypeProto>, Uuid>,
}

impl Representations {
    pub fn new() -> Representations {
        Representations {
            builtin_functions: Default::default(),
            function_representations: Default::default(),
            type_ids: Default::default(),
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum FunctionRepresentation {
    Constant(String),
    FunctionCall(String),
    Unary(String),
    Binary(String),
}

impl FunctionRepresentation {
    pub fn name(&self) -> &String {
        match self {
            FunctionRepresentation::Constant(n) => n,
            FunctionRepresentation::FunctionCall(n) => n,
            FunctionRepresentation::Unary(n) => n,
            FunctionRepresentation::Binary(n) => n,
        }
    }
}

pub fn find_for_functions<'a>(function_representations: &mut HashMap<Rc<FunctionHead>, FunctionRepresentation>, names: &HashMap<Uuid, String>, functions: impl Iterator<Item=&'a Box<FunctionImplementation>>) {
    // TODO EVERYTHING without side effects can be a constant.
    //  We should figure out whether we have side effects and then act accordingly.
    // TODO We'll also need to sort functions by who calls who.

    for function in functions {
        function_representations.insert(Rc::clone(&function.head), FunctionRepresentation::FunctionCall(names[&function.head.function_id].clone()));
    }
}
