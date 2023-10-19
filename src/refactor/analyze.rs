use linked_hash_set::LinkedHashSet;
use std::rc::Rc;
use crate::program::computation_tree::ExpressionOperation;
use crate::program::functions::FunctionHead;
use crate::program::global::FunctionImplementation;

pub fn gather_callees(implementation: &FunctionImplementation) -> LinkedHashSet<Rc<FunctionHead>> {
    let mut callees = LinkedHashSet::new();

    // TODO Generic function calls would break this logic
    for expression_id in implementation.expression_forest.deep_children(implementation.root_expression_id) {
        let expression_op = &implementation.expression_forest.operations[&expression_id];
        match expression_op {
            ExpressionOperation::FunctionCall(f) => {
                callees.insert(Rc::clone(&f.function));
            }
            ExpressionOperation::PairwiseOperations { .. } => {
                todo!()
            }
            _ => {}
        }
    }

    callees
}
