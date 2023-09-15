use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use crate::program::allocation::ObjectReference;
use crate::program::calls::FunctionBinding;
use crate::program::generics::GenericAlias;

pub type ExpressionID = GenericAlias;

#[derive(Clone)]
pub enum Statement {
    VariableAssignment(Rc<ObjectReference>, ExpressionID),
    Expression(ExpressionID),
    Return(Option<ExpressionID>),
}

#[derive(Clone)]
pub enum ExpressionOperation {
    FunctionCall(Rc<FunctionBinding>),
    PairwiseOperations { calls: Vec<Rc<FunctionBinding>> },
    VariableLookup(Rc<ObjectReference>),
    ArrayLiteral,
    StringLiteral(String),
    Block(Vec<Box<Statement>>),
}

// TODO We should integrate statements into the tree somehow, so it can be traversed fully automatically.
//  One solution might be to use blocks' arguments to link to each statement, and then to link only the
//  top block or expression in FunctionImplementation.
#[derive(Clone)]
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
