use std::collections::HashSet;
use std::rc::Rc;
use itertools::Itertools;
use uuid::Uuid;
use crate::linker::interface::{FunctionHead, FunctionInterface};
use crate::program::allocation::ObjectReference;
use crate::program::computation_tree::ExpressionOperation;
use crate::program::global::FunctionImplementation;

pub fn swizzle_retaining_parameters(function: &FunctionImplementation, unused: &HashSet<Rc<ObjectReference>>) -> Vec<usize> {
    function.parameter_locals.iter().enumerate()
        .filter_map(|(idx, local)| (!unused.contains(local)).then(|| idx))
        .collect_vec()
}

pub fn find_unused_locals(function: &FunctionImplementation) -> HashSet<Rc<ObjectReference>> {
    let mut unused = HashSet::from_iter(function.locals_names.keys().cloned());

    for operation in function.expression_forest.operations.values() {
        match operation {
            ExpressionOperation::GetLocal(local) => _ = unused.remove(local),
            _ => {},
        }
    }

    return unused
}

pub fn swizzle_parameters(implementation: &FunctionImplementation, swizzle: &Vec<usize>) -> Box<FunctionImplementation> {
    // TODO We may be able to remove some generics and requirements.
    let new_head = FunctionHead::new(Rc::new(FunctionInterface {
        parameters: swizzle.iter().map(|idx| implementation.head.interface.parameters[*idx].clone()).collect_vec(),
        return_type: implementation.head.interface.return_type.clone(),
        requirements: implementation.head.interface.requirements.clone(),
        generics: implementation.head.interface.generics.clone(),
    }), implementation.head.function_type.clone());

    let mut locals_names = implementation.locals_names.clone();
    for (idx, parameter) in  implementation.parameter_locals.iter().enumerate() {
        if swizzle.contains(&idx) {
            continue
        }

        locals_names.remove(parameter);
    }

    Box::new(FunctionImplementation {
        implementation_id: Uuid::new_v4(),
        head: Rc::clone(&new_head),
        requirements_assumption: implementation.requirements_assumption.clone(),
        root_expression_id: implementation.root_expression_id,
        expression_forest: implementation.expression_forest.clone(),
        type_forest: implementation.type_forest.clone(),
        parameter_locals: swizzle.iter().map(|idx| implementation.parameter_locals[*idx].clone()).collect_vec(),
        locals_names,
    })
}
