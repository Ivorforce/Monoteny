use std::collections::HashMap;
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use crate::linker::interface::FunctionHead;
use crate::program::calls::FunctionBinding;
use crate::program::computation_tree::{ExpressionID, ExpressionOperation, truncate_tree};
use crate::program::global::FunctionImplementation;
use crate::program::traits::RequirementsFulfillment;
use crate::refactor::InlineHint;

pub fn try_inline(implementation: &FunctionImplementation) -> Option<InlineHint> {
    if !implementation.requirements_assumption.conformance.is_empty() {
        // TODO We can probably inline that too, but it would require extracting the abstract function
        //  from the caller. This is non trivial and not needed for now.
        return None;
    }

    match (&implementation.expression_forest.operations[&implementation.root_expression_id], &implementation.expression_forest.arguments[&implementation.root_expression_id].as_slice()) {
        (ExpressionOperation::Block, []) => Some(InlineHint::NoOp),
        // While this might result in a return where one wasn't expected,
        // any receiver that can handle a void return won't do anything with the return value.
        // Basically the receiver is guaranteed to be a block or a no-return function. So it's fine.
        (ExpressionOperation::Block, [arg]) => get_trivial_expression_call_target(arg, implementation),
        (ExpressionOperation::Return, [arg]) => get_trivial_expression_call_target(arg, implementation),
        _ => get_trivial_expression_call_target(&implementation.root_expression_id, implementation),
    }
}

pub fn get_trivial_expression_call_target(expression_id: &ExpressionID, implementation: &FunctionImplementation) -> Option<InlineHint> {
    // Anything that's not a 'trivial call' or argument return should not be inlined (here).
    // --> It may be an explicit constant or function that makes the code more readable.
    // --> We are only concerned about readability. If folding is useful for performance, the target compiler shall do it.
    // If requested, the 'constant fold' part with run an interpreter to inline those functions anyway, replacing the calls with constant values.
    match &implementation.expression_forest.operations[expression_id] {
        ExpressionOperation::FunctionCall(f) => {
            if !f.requirements_fulfillment.is_empty() {
                return None
            }

            let replace_args: Vec<_> = implementation.expression_forest.arguments[expression_id].iter().map(|arg| {
                match &implementation.expression_forest.operations[arg] {
                    ExpressionOperation::GetLocal(v) => {
                        if let Some(idx) = implementation.parameter_locals.iter().position(|ref_| ref_ == v) {
                            return Some(idx)
                        }
                    }
                    _ => { },
                }

                None
            }).collect::<Option<_>>()?;
            if replace_args.iter().duplicates().next().is_some() {
                // If we use the same argument twice, we cannot trivially be inlined because arguments
                //  would have to be copied to variables first - otherwise, we reference the same
                //  expression twice in the expression forest.
                return None
            }

            return Some(InlineHint::ReplaceCall(Rc::clone(&f.function), replace_args))
        },
        ExpressionOperation::GetLocal(v) => {
            if let Some(idx) = implementation.parameter_locals.iter().position(|ref_| ref_ == v) {
                return Some(InlineHint::YieldParameter(idx))
            }
        }
        _ => {},
    }

    None
}

pub fn inline_calls(implementation: &mut Box<FunctionImplementation>, hints: &HashMap<Rc<FunctionHead>, InlineHint>) {
    let expression_forest = &mut implementation.expression_forest;
    for expression_id in expression_forest.deep_children(implementation.root_expression_id) {
        guard!(let Some(operation) = expression_forest.operations.get(&expression_id) else {
            // We have been truncated meanwhile!
            continue;
        });

        match operation {
            ExpressionOperation::FunctionCall(f) => {
                if let Some(inline_hint) = hints.get(&f.function) {
                    match inline_hint {
                        InlineHint::ReplaceCall(target_function, idxs) => {
                            let operation = expression_forest.operations.get_mut(&expression_id).unwrap();
                            *operation = ExpressionOperation::FunctionCall(Rc::new(FunctionBinding {
                                function: Rc::clone(&target_function),
                                // The requirements fulfillment can be empty because otherwise it wouldn't have been inlined.
                                requirements_fulfillment: RequirementsFulfillment::empty(),
                            }));
                            let arguments = expression_forest.arguments.get_mut(&expression_id).unwrap();
                            let arguments_before = arguments.clone();
                            *arguments = idxs.iter().map(|idx| arguments[*idx]).collect_vec();
                            truncate_tree(arguments_before, arguments.iter().cloned().collect(), expression_forest);
                        }
                        InlineHint::YieldParameter(idx) => {
                            let arguments_before = expression_forest.arguments[&expression_id].clone();
                            let replacement_id = arguments_before[*idx];
                            let replacement_operation = expression_forest.operations.remove(&replacement_id).unwrap();
                            let replacement_arguments = expression_forest.arguments.remove(&replacement_id).unwrap();

                            let operation = expression_forest.operations.get_mut(&expression_id).unwrap();
                            *operation = replacement_operation;
                            let arguments = expression_forest.arguments.get_mut(&expression_id).unwrap();
                            *arguments = replacement_arguments;
                            truncate_tree(arguments_before, [replacement_id].into_iter().collect(), expression_forest);
                        },
                        InlineHint::NoOp => {
                            todo!("Should do this in another loop and remove if it's in a block, otherwise throw")
                        },
                    }
                }
            }
            _ => {}
        }
    }
}
