use std::fmt::Debug;
use std::rc::Rc;

use crate::program::allocation::ObjectReference;
use crate::program::calls::FunctionBinding;
use crate::program::generics::GenericAlias;
use crate::util::graphs::node_tree::NodeTree;

pub type ExpressionID = GenericAlias;


#[derive(Clone, PartialEq, Eq, Debug)]
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

    // TODO We can remove these operations if we just add a getter and setter for every global.
    GetLocal(Rc<ObjectReference>),
    SetLocal(Rc<ObjectReference>),

    // 0 arguments if no return type is set, otherwise 1
    Return,

    FunctionCall(Rc<FunctionBinding>),
    PairwiseOperations { calls: Vec<Rc<FunctionBinding>> },

    // TODO This is required because it has a variable number of arguments (its elements).
    //  This is not supported in functions otherwise, and we'd have to make an exception.
    //  Which might be fair in the future, but for now it's not a pressing concern.
    ArrayLiteral,
    StringLiteral(String),
}

pub type ExpressionTree = NodeTree<ExpressionID, ExpressionOperation>;
