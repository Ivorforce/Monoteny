use std::collections::HashMap;
use std::rc::Rc;

use itertools::Itertools;

use crate::program::calls::{FunctionBinding, resolve_binding};
use crate::program::expression_tree::{ExpressionID, ExpressionOperation};
use crate::program::functions::FunctionHead;
use crate::program::global::FunctionImplementation;
use crate::program::traits::RequirementsFulfillment;

#[derive(Clone, Debug)]
pub enum InlineHint {
    ReplaceCall(Rc<FunctionHead>, Vec<usize>),
    YieldParameter(usize),
    NoOp,
}

pub fn try_inline(implementation: &FunctionImplementation) -> Option<InlineHint> {
    if !implementation.requirements_assumption.conformance.is_empty() {
        // TODO We can probably inline that too, but it would require extracting the abstract function
        //  from the caller. This is non trivial and not needed for now.
        return None;
    }

    match (&implementation.expression_tree.values[&implementation.expression_tree.root], &implementation.expression_tree.children[&implementation.expression_tree.root].as_slice()) {
        (ExpressionOperation::Block, []) => Some(InlineHint::NoOp),
        // While this might result in a return where one wasn't expected,
        // any receiver that can handle a void return won't do anything with the return value.
        // Basically the receiver is guaranteed to be a block or a no-return function. So it's fine.
        (ExpressionOperation::Block, [arg]) => get_trivial_expression_call_target(arg, implementation),
        (ExpressionOperation::Return, [arg]) => get_trivial_expression_call_target(arg, implementation),
        _ => get_trivial_expression_call_target(&implementation.expression_tree.root, implementation),
    }
}

pub fn get_trivial_expression_call_target(expression_id: &ExpressionID, implementation: &FunctionImplementation) -> Option<InlineHint> {
    // Anything that's not a 'trivial call' or argument return should not be inlined (here).
    // --> It may be an explicit constant or function that makes the code more readable.
    // --> We are only concerned about readability. If folding is useful for performance, the target compiler shall do it.
    // If requested, the 'constant fold' part with run an interpreter to inline those functions anyway, replacing the calls with constant values.
    match &implementation.expression_tree.values[expression_id] {
        ExpressionOperation::FunctionCall(f) => {
            let replace_args: Vec<_> = implementation.expression_tree.children[expression_id].iter().map(|arg| {
                match &implementation.expression_tree.values[arg] {
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

            if !f.requirements_fulfillment.is_empty() {
                // TODO We just need to lift the requirements fulfillment upwards, it may still be possible!
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

pub fn inline_calls(
    implementation: &mut Box<FunctionImplementation>,
    optimizations: &HashMap<Rc<FunctionBinding>, Rc<FunctionHead>>,
    hints: &HashMap<Rc<FunctionHead>, InlineHint>,
) {
    let expression_forest = &mut implementation.expression_tree;

    'expression: for expression_id in expression_forest.deep_children(expression_forest.root) {
        // Essentially, we run through the expression tree. When we change an operation,
        //  we run through it again because there may be more mappings.
        'inline: loop {
            let Some(operation) = expression_forest.values.get(&expression_id) else {
                // We have been truncated meanwhile!
                continue 'expression;
            };

            // TODO If any parameters do I/O, we need to group them together as part of a new block
            //  and run them successively.
            match operation {
                ExpressionOperation::FunctionCall(f) => {
                    if let Some(optimized_head) = optimizations.get(&resolve_binding(f, &implementation.type_forest)) {
                        let operation = expression_forest.values.get_mut(&expression_id).unwrap();
                        *operation = ExpressionOperation::FunctionCall(Rc::new(FunctionBinding {
                            function: Rc::clone(&optimized_head),
                            // TODO If we're not fully monomorphized, this may not be empty.
                            requirements_fulfillment: RequirementsFulfillment::empty(),
                        }));
                        continue 'inline
                    }

                    if let Some(inline_hint) = hints.get(&f.function) {
                        match inline_hint {
                            InlineHint::ReplaceCall(target_function, idxs) => {
                                let operation = expression_forest.values.get_mut(&expression_id).unwrap();
                                *operation = ExpressionOperation::FunctionCall(Rc::new(FunctionBinding {
                                    function: Rc::clone(&target_function),
                                    // TODO If we're not monomorphized, this may not be empty.
                                    requirements_fulfillment: RequirementsFulfillment::empty(),
                                }));
                                expression_forest.swizzle_arguments(expression_id, idxs);
                                continue 'inline
                            }
                            InlineHint::YieldParameter(idx) => {
                                expression_forest.inline(expression_id, *idx);
                                continue 'inline
                            },
                            InlineHint::NoOp => {
                                todo!("Should do this in another loop and remove if it's in a block, otherwise throw");
                                continue 'inline
                            },
                        }
                    }
                }
                ExpressionOperation::PairwiseOperations { .. } => {
                    todo!()
                }
                _ => {},
            }

            continue 'expression
        }
    }
}
