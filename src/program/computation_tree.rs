use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::Debug;
use std::rc::Rc;
use itertools::Itertools;
use crate::program::allocation::ObjectReference;
use crate::program::calls::FunctionBinding;
use crate::program::generics::GenericAlias;
use crate::util::iter::omega;
use crate::util::vec;

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

// TODO We should integrate statements into the tree somehow, so it can be traversed fully automatically.
//  One solution might be to use blocks' arguments to link to each statement, and then to link only the
//  top block or expression in FunctionImplementation.
#[derive(Clone)]
pub struct ExpressionTree {
    /// Will be set for every expression ID
    pub arguments: HashMap<ExpressionID, Vec<ExpressionID>>,
    /// Will be set for every expression ID
    pub references: HashMap<ExpressionID, ExpressionID>,
    /// Might not be set for a while
    pub operations: HashMap<ExpressionID, ExpressionOperation>,
}

impl ExpressionTree {
    pub fn new() -> ExpressionTree {
        ExpressionTree {
            operations: HashMap::new(),
            arguments: HashMap::new(),
            references: Default::default(),
        }
    }

    pub fn deep_children(&self, start: ExpressionID) -> Vec<ExpressionID> {
        omega([start].into_iter(), |e| self.arguments[e].iter().cloned()).collect_vec()
    }

    pub fn truncate(&mut self, mut trim: Vec<ExpressionID>) {
        while let Some(current) = trim.pop() {
            if let Entry::Occupied(o) = self.operations.entry(current) {
                match o.get() {
                    ExpressionOperation::Block => self.arguments.get_mut(&current).unwrap().retain(|a| self.operations.contains_key(a)),
                    _ => {
                        o.remove();
                        trim.push(self.references.remove(&current).unwrap());
                        trim.extend(self.arguments.remove(&current).unwrap());
                    }
                }
            }
        }
    }

    pub fn truncate_down(&mut self, mut include: Vec<ExpressionID>) {
        while let Some(current) = include.pop() {
            // It's enough to remove arguments and operations.
            // The type forest may still contain orphans, but that's ok. And our type doesn't change
            //  from inlining.
            self.operations.remove(&current);
            self.references.remove(&current);
            include.extend(self.arguments.remove(&current).unwrap());
        }
    }

    pub fn swizzle_arguments(&mut self, expression_id: ExpressionID, swizzle: &Vec<usize>) {
        let args = self.arguments.get_mut(&expression_id).unwrap();
        let removed = vec::swizzle(args, swizzle);
        self.truncate_down(removed);
    }

    pub fn inline(&mut self, expression_id: ExpressionID, parameter_idx: usize) {
        let mut arguments_before = self.arguments[&expression_id].clone();

        let replacement_id = arguments_before.remove(parameter_idx);
        let replacement_operation = self.operations.remove(&replacement_id).unwrap();
        let replacement_arguments = self.arguments.remove(&replacement_id).unwrap();

        for arg in replacement_arguments.iter() {
            self.references.insert(*arg, expression_id);
        }

        let operation = self.operations.get_mut(&expression_id).unwrap();
        *operation = replacement_operation;

        let arguments = self.arguments.get_mut(&expression_id).unwrap();
        *arguments = replacement_arguments;

        self.truncate_down(arguments_before);
    }
}
