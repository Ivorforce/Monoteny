pub mod constant_folding;
pub mod monomorphize;

use std::collections::{HashMap, HashSet};
use std::ops::DerefMut;
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use crate::refactor::constant_folding::InlineHint;
use crate::linker::interface::FunctionHead;
use crate::program::calls::FunctionBinding;
use crate::program::computation_tree::{ExpressionID, ExpressionOperation, truncate_tree};
use crate::program::global::FunctionImplementation;
use crate::program::traits::RequirementsFulfillment;
use crate::util::iter::omega;
use crate::util::multimap::insert_into_multimap;

pub struct Refactor<'a> {
    pub implementation_by_head: HashMap<Rc<FunctionHead>, &'a mut FunctionImplementation>,
    pub dependents: HashMap<Rc<FunctionHead>, HashSet<Rc<FunctionHead>>>,

    pub forbid_interface_changes: HashSet<Rc<FunctionHead>>,
    pub inline_hints: HashMap<Rc<FunctionHead>, InlineHint>,
}

impl<'a> Refactor<'a> {
    pub fn new() -> Refactor<'a> {
        Refactor {
            implementation_by_head: Default::default(),
            dependents: Default::default(),
            forbid_interface_changes: Default::default(),
            inline_hints: Default::default(),
        }
    }

    pub fn add(&mut self, implementation: &'a mut FunctionImplementation, allow_inline: bool) {
        let head = Rc::clone(&implementation.head);

        if !allow_inline {
            self.forbid_interface_changes.insert(Rc::clone(&head));
        }

        for dependency in gather_dependencies(implementation) {
            insert_into_multimap(&mut self.dependents, dependency, Rc::clone(&implementation.head));
        }

        self.implementation_by_head.insert(Rc::clone(&head), implementation);

        if !self.inline_hints.is_empty() {
            // In case any calls have already been inlined before this implementation was added.
            self.inline_calls(&head);
        }
    }

    pub fn try_inline(&mut self, head: &Rc<FunctionHead>) -> bool {
        if self.forbid_interface_changes.contains(head) {
            return false
        }

        guard!(let Some(imp) = self.implementation_by_head.get_mut(head) else {
            return false
        });

        guard!(let Some(hint) = try_inline(imp) else {
            return false
        });

        self._inline_cascade(head, hint);

        for dependent in self.dependents.get(head).iter().flat_map(|x| x.iter()).cloned().collect_vec() {
            self.inline_calls(&dependent);
        }

        return true
    }

    pub fn inline_calls(&mut self, head: &Rc<FunctionHead>) {
        let implementation = self.implementation_by_head.get_mut(head).unwrap();

        let expression_forest = &mut implementation.expression_forest;
        for expression_id in expression_forest.deep_children(implementation.root_expression_id) {
            guard!(let Some(operation) = expression_forest.operations.get(&expression_id) else {
                // We have been truncated meanwhile!
                continue;
            });

            match operation {
                ExpressionOperation::FunctionCall(f) => {
                    if let Some(inline_hint) = self.inline_hints.get(&f.function) {
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

        // TODO We don't know what calls we removed, so we don't know on whose dependents we still are.
        //  This isn't a problem, just a bit ínefficient.
        for dependency in gather_dependencies(implementation) {
            insert_into_multimap(&mut self.dependents, dependency, Rc::clone(&implementation.head));
        }
    }

    fn _inline_cascade(&mut self, head: &Rc<FunctionHead>, hint: InlineHint) {
        let all_affected = omega([(head, hint)].into_iter(), |(head, hint)| {
            return self.dependents.get(*head).iter().flat_map(|x| x.iter())
                .filter_map(|dependent| self.inline_hints.remove(dependent).map(|hint| (dependent, hint)))
                .collect_vec().into_iter()
        });

        for (head, hint) in all_affected {
            match &hint {
                InlineHint::ReplaceCall(target, arg_idxs) => {
                    match self.inline_hints.get(target) {
                        None => self.inline_hints.insert(Rc::clone(&head), hint),
                        Some(InlineHint::ReplaceCall(target_head, target_arg_idxs)) => {
                            self.inline_hints.insert(
                                Rc::clone(&head),
                                InlineHint::ReplaceCall(Rc::clone(target_head), target_arg_idxs.iter().map(|idx| arg_idxs[*idx].clone()).collect_vec())
                            )
                        }
                        Some(InlineHint::YieldParameter(yield_idx)) => {
                            // Same as above here.
                            assert_eq!(*yield_idx, 0);
                            self.inline_hints.insert(Rc::clone(&head), InlineHint::YieldParameter(arg_idxs[*yield_idx]))
                        }
                        Some(other_hint) => self.inline_hints.insert(Rc::clone(&head), other_hint.clone()),
                    }
                }
                _ => self.inline_hints.insert(Rc::clone(&head), hint)
            };
        }
    }
}

pub fn gather_dependencies(implementation: &FunctionImplementation) -> HashSet<Rc<FunctionHead>> {
    let mut dependencies = HashSet::new();

    // First get all dependencies
    for expression_op in implementation.expression_forest.operations.values() {
        match expression_op {
            ExpressionOperation::FunctionCall(f) => {
                dependencies.insert(Rc::clone(&f.function));
            }
            ExpressionOperation::PairwiseOperations { .. } => {
                todo!()
            }
            _ => {}
        }
    }

    dependencies
}

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
