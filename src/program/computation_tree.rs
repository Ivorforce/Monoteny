use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use guard::guard;
use uuid::Uuid;
use crate::parser::abstract_syntax::Function;
use crate::program::allocation::{Mutability, ObjectReference, Reference};
use crate::program::types::TypeProto;

use crate::program::builtins::Builtins;
use crate::program::functions::{FunctionPointer, FunctionInterface, ParameterKey};
use crate::program::generics::{GenericAlias, TypeForest};
use crate::program::primitives;
use crate::program::traits::{Trait, TraitBinding, TraitConformanceDeclaration, TraitConformanceRequirement};

pub type ExpressionID = GenericAlias;

pub enum Statement {
    VariableAssignment(Rc<ObjectReference>, ExpressionID),
    Expression(ExpressionID),
    Return(Option<ExpressionID>),
}

pub enum ExpressionOperation {
    FunctionCall { function: Rc<FunctionPointer>, argument_targets: Vec<Rc<ObjectReference>>, binding: Box<TraitBinding> },
    PairwiseOperations { functions: Vec<Rc<FunctionInterface>> },
    VariableLookup(Rc<ObjectReference>),
    ArrayLiteral,
    StringLiteral(String),
}

pub struct ExpressionForest {
    /// Will be set for every expression ID
    pub arguments: HashMap<ExpressionID, Vec<ExpressionID>>,
    /// Might not be set for a while
    pub operations: HashMap<ExpressionID, ExpressionOperation>,
}

impl ExpressionForest {
    pub fn new() -> ExpressionForest {
        ExpressionForest {
            operations: HashMap::new(),
            arguments: HashMap::new(),
        }
    }
}

impl Debug for Statement {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::VariableAssignment(target, expression_id) => {
                write!(fmt, "ASSIGN {} to {}", &target.id, expression_id)
            }
            Statement::Expression(expression) => {
                write!(fmt, "RUN {}", expression)
            }
            Statement::Return(expression) => {
                match expression {
                    None => write!(fmt, "RETURN"),
                    Some(expression) => write!(fmt, "RETURN {}", expression),
                }
            }
        }
    }
}
