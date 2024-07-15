use std::collections::HashSet;
use std::rc::Rc;

use crate::error::RResult;
use crate::program::functions::{FunctionHead, FunctionRepresentation};

/// Reference to a multiplicity of functions, usually resolved when attempting to call
#[derive(Clone, PartialEq, Eq)]
pub struct FunctionOverload {
    pub functions: HashSet<Rc<FunctionHead>>,
    // Note: If representation is NOT an implicit, the functions within are getters.
    pub representation: FunctionRepresentation,
}

impl FunctionOverload {
    pub fn from(function: &Rc<FunctionHead>, representation: FunctionRepresentation) -> Rc<FunctionOverload> {
        Rc::new(FunctionOverload {
            functions: HashSet::from([Rc::clone(function)]),
            representation,
        })
    }

    pub fn adding_function(&self, function: &Rc<FunctionHead>) -> RResult<Rc<FunctionOverload>> {
        Ok(Rc::new(FunctionOverload {
            functions: self.functions.iter()
                .chain([function])
                .map(Rc::clone)
                .collect(),
            representation: self.representation.clone(),
        }))
    }
}
