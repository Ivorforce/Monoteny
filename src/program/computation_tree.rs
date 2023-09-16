use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use crate::program::allocation::ObjectReference;
use crate::program::calls::FunctionBinding;
use crate::program::generics::GenericAlias;

pub type ExpressionID = GenericAlias;


#[derive(Clone)]
pub enum ExpressionOperation {
    // TODO Blocks are a tough one to transpile as no language supports yields.
    //  They will probably have to be inlined as a variable, like e.g.:
    //  var x: Int;
    //  for i in 0 ..< 1 {
    //      ...
    //      // yield 5;
    //      x = 5;
    //      break;
    //  }
    //  This syntax, while stupid, is at least supported in pretty much every language.
    Block,

    VariableAssignment(Rc<ObjectReference>),
    // 0 arguments if no return type is set, otherwise 1
    Return,

    FunctionCall(Rc<FunctionBinding>),
    PairwiseOperations { calls: Vec<Rc<FunctionBinding>> },
    VariableLookup(Rc<ObjectReference>),
    ArrayLiteral,
    StringLiteral(String),
}

// TODO We should integrate statements into the tree somehow, so it can be traversed fully automatically.
//  One solution might be to use blocks' arguments to link to each statement, and then to link only the
//  top block or expression in FunctionImplementation.
#[derive(Clone)]
pub struct ExpressionTree {
    /// Will be set for every expression ID
    pub arguments: HashMap<ExpressionID, Vec<ExpressionID>>,
    /// Might not be set for a while
    pub operations: HashMap<ExpressionID, ExpressionOperation>,
}

impl ExpressionTree {
    pub fn new() -> ExpressionTree {
        ExpressionTree {
            operations: HashMap::new(),
            arguments: HashMap::new(),
        }
    }
}
