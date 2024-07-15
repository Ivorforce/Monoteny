use std::rc::Rc;

use linked_hash_set::LinkedHashSet;

use crate::program::expression_tree::ExpressionOperation;
use crate::program::functions::{FunctionBinding, FunctionImplementation, resolve_binding};

pub fn gather_callees(implementation: &FunctionImplementation) -> LinkedHashSet<Rc<FunctionBinding>> {
    let mut callees = LinkedHashSet::new();

    // TODO Generic function calls would break this logic
    for expression_id in implementation.expression_tree.deep_children(implementation.expression_tree.root) {
        let expression_op = &implementation.expression_tree.values[&expression_id];
        match expression_op {
            ExpressionOperation::FunctionCall(f) => {
                callees.insert(resolve_binding(f, &implementation.type_forest));
            }
            ExpressionOperation::PairwiseOperations { .. } => {
                todo!()
            }
            _ => {}
        }
    }

    callees
}
