use std::collections::HashSet;
use std::rc::Rc;

use itertools::Itertools;

use crate::program::allocation::ObjectReference;
use crate::program::expression_tree::ExpressionOperation;
use crate::program::functions::{FunctionHead, FunctionInterface};
use crate::program::global::FunctionImplementation;

pub fn swizzle_retaining_parameters(function: &FunctionImplementation, removed: &HashSet<Rc<ObjectReference>>) -> Vec<usize> {
    function.parameter_locals.iter().enumerate()
        .filter_map(|(idx, local)| (!removed.contains(local)).then(|| idx))
        .collect_vec()
}

pub fn find_unused_locals(function: &FunctionImplementation) -> HashSet<Rc<ObjectReference>> {
    let mut unused = HashSet::from_iter(function.locals_names.keys().cloned());

    for operation in function.expression_tree.values.values() {
        match operation {
            ExpressionOperation::GetLocal(local) => _ = unused.remove(local),
            _ => {},
        }
    }

    return unused
}

pub fn remove_locals(implementation: &mut FunctionImplementation, removed_locals: &HashSet<Rc<ObjectReference>>) -> Option<Vec<usize>> {
    let changes_interface = removed_locals.iter().any(|l| implementation.parameter_locals.contains(l));

    let mut expression_forest = &mut implementation.expression_tree;
    // TODO Also truncate removed from type forest
    let mut type_forest = &mut implementation.type_forest;

    for expression_id in expression_forest.values.keys().cloned().collect_vec() {
        let Some(operation) = expression_forest.values.get(&expression_id) else {
            continue  // Already trimmed
        };

        match operation {
            ExpressionOperation::GetLocal(local) => {
                if removed_locals.contains(local) {
                    expression_forest.truncate_up_and_down(vec![expression_id], |op| op == &ExpressionOperation::Block);
                }
            }
            ExpressionOperation::SetLocal(local) => {
                if removed_locals.contains(local) {
                    expression_forest.truncate_up_and_down(vec![expression_id], |op| op == &ExpressionOperation::Block);
                }
            }
            _ => {},
        }
    }

    implementation.locals_names = implementation.locals_names.drain()
        .filter(|(key, value)| !removed_locals.contains(key))
        .collect();

    if changes_interface {
        let swizzle = swizzle_retaining_parameters(implementation, removed_locals);

        // TODO We may be able to remove some generics and requirements.
        let new_head = FunctionHead::new(Rc::new(FunctionInterface {
            parameters: swizzle.iter().map(|idx| implementation.head.interface.parameters[*idx].clone()).collect_vec(),
            return_type: implementation.head.interface.return_type.clone(),
            requirements: implementation.head.interface.requirements.clone(),
            generics: implementation.head.interface.generics.clone(),
        }), implementation.head.function_type.clone());

        implementation.head = new_head;
        implementation.parameter_locals = swizzle.iter().map(|idx| implementation.parameter_locals[*idx].clone()).collect_vec();

        return Some(swizzle)
    }
    else {
        return None
    }
}
