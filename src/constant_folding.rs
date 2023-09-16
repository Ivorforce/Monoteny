use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use crate::program::allocation::ObjectReference;
use crate::program::calls::FunctionBinding;
use crate::program::computation_tree::{ExpressionTree, ExpressionID, ExpressionOperation};
use crate::program::functions::FunctionHead;
use crate::program::global::FunctionImplementation;
use crate::program::traits::RequirementsFulfillment;
use crate::util::multimap::insert_into_multimap;

pub struct ConstantFold {
    pub implementation_by_head: HashMap<Rc<FunctionHead>, Box<FunctionImplementation>>,

    pub dependents: HashMap<Rc<FunctionHead>, HashSet<Rc<FunctionHead>>>,

    pub forbid_inline: HashSet<Rc<FunctionHead>>,
    pub inline_hints: HashMap<Rc<FunctionHead>, InlineHint>,
}

#[derive(Clone, Debug)]
pub enum InlineHint {
    ReplaceCall(Rc<FunctionHead>, usize),
    YieldParameter(usize),
    GlobalLookup(Rc<ObjectReference>),
    NoOp,
}

/// For now, the constant folder is only capable of inlining trivial functions:
/// - those that return a parameter
/// - those that call another function with one argument
/// - those that lookup a global variable (eg function reference)
/// - those that do nothing
///
impl ConstantFold {
    pub fn new() -> ConstantFold {
        ConstantFold {
            implementation_by_head: Default::default(),
            dependents: Default::default(),
            forbid_inline: Default::default(),
            inline_hints: Default::default(),
        }
    }

    pub fn add(&mut self, mut implementation: Box<FunctionImplementation>, allow_inline: bool) {
        self.gather_dependencies(&mut implementation);

        if !allow_inline {
            self.forbid_inline.insert(Rc::clone(&implementation.head));
        }

        if let Some(hint) = self.inline_calls(&mut implementation) {
            let head = Rc::clone(&implementation.head);
            // Need to do this first because self.inline() may look us up recursively
            self.implementation_by_head.insert(Rc::clone(&head), implementation);
            self.inline(&head, hint);
        }
        else {
            self.implementation_by_head.insert(Rc::clone(&implementation.head), implementation);
        }
    }

    fn gather_dependencies(&mut self, implementation: &FunctionImplementation) {
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

        for dependency in dependencies {
            insert_into_multimap(&mut self.dependents, dependency, Rc::clone(&implementation.head));
        }
    }

    pub fn inline(&mut self, head: &Rc<FunctionHead>, hint: InlineHint) {
        self._inline_cascade(head, hint);

        guard!(let Some(todo) = self.dependents.remove(head) else {
            return;  // Nobody calls us (yet)
        });
        let mut todo = todo.into_iter().collect_vec();

        while let Some(head) = todo.pop() {
            if self.inline_hints.contains_key(&head) {
                // We already inlined this one
                continue
            }

            // TODO We remove it temporarily because otherwise we need to borrow self as mutable.
            let mut implementation = self.implementation_by_head.remove(&head).unwrap();

            if let Some(hint) = self.inline_calls(&mut implementation) {
                self._inline_cascade(&head, hint);
                // todo we may do the same function twice without changes.
                // Optimize when need be!
                if let Some(dependents) = self.dependents.remove(&head) {
                    todo.extend(dependents);
                }
            }

            self.implementation_by_head.insert(head, implementation);
        }
    }

    fn _inline_cascade(&mut self, head: &Rc<FunctionHead>, hint: InlineHint) {
        match &hint {
            InlineHint::ReplaceCall(target, arg_idx) => {
                match self.inline_hints.get(target) {
                    None => self.inline_hints.insert(Rc::clone(head), hint),
                    Some(InlineHint::ReplaceCall(new_head, other_arg_idx)) => {
                        // We could only have been inlined if we performed a call without swizzle (just one of our arguments).
                        // So the other function can only have had exactly one function.
                        assert_eq!(*other_arg_idx, 0);
                        self.inline_hints.insert(Rc::clone(&head), InlineHint::ReplaceCall(Rc::clone(new_head), *arg_idx))
                    }
                    Some(InlineHint::YieldParameter(other_arg_idx)) => {
                        // Same as above here.
                        assert_eq!(*other_arg_idx, 0);
                        self.inline_hints.insert(Rc::clone(&head), InlineHint::YieldParameter(*arg_idx))
                    }
                    Some(other_hint) => self.inline_hints.insert(Rc::clone(head), other_hint.clone()),
                }
            }
            _ => self.inline_hints.insert(Rc::clone(head), hint)
        };

        // This isn't efficient but it's easy to write
        for dependent in self.dependents.get(head).cloned().unwrap_or(HashSet::new()) {
            if let Some(inline_hint) = self.inline_hints.remove(&dependent) {
                // Re-insert the dependent with the same cascading rules as above
                // - transitive rules ensure we never accidentally call a function that's already inlined.
                self._inline_cascade(&dependent, inline_hint);
            }
        }
    }

    pub fn inline_calls(&mut self, implementation: &mut FunctionImplementation) -> Option<InlineHint> {
        for expression_id in implementation.expression_forest.operations.keys().cloned().collect_vec() {
            self.inline_expression(implementation, expression_id);
        }

        // Reverse iteration allows us to remove objects without indexes getting invalidated.
        for (expression_id, operation) in implementation.expression_forest.operations.iter() {
            match operation {
                ExpressionOperation::Block => {
                    let arguments = implementation.expression_forest.arguments.get_mut(expression_id).unwrap();
                    arguments.retain(|a| implementation.expression_forest.operations.contains_key(a));
                }
                _ => {}
            }
        }

        // TODO We don't know what calls we removed, so we don't know on whose dependents we still are.
        // This isn't a problem, just a bit ínefficient.
        self.gather_dependencies(implementation);

        if self.forbid_inline.contains(&implementation.head) {
            None
        }
        else {
            try_inline(implementation)
        }
    }

    fn inline_expression(&mut self, implementation: &mut FunctionImplementation, expression_id: ExpressionID) {
        guard!(let Some(operation) = implementation.expression_forest.operations.get(&expression_id) else {
            return  // We may have truncated this WHILE iterating the forest.
        });
        let arguments = &implementation.expression_forest.arguments[&expression_id].clone();

        match operation {
            ExpressionOperation::FunctionCall(f) => {
                if let Some(inline_hint) = self.inline_hints.get(&f.function) {
                    match inline_hint {
                        InlineHint::ReplaceCall(target_function, idx) => {
                            implementation.expression_forest.operations.insert(expression_id, ExpressionOperation::FunctionCall(Rc::new(FunctionBinding {
                                function: Rc::clone(&target_function),
                                // The requirements fulfillment can be empty because otherwise it wouldn't have been inlined.
                                requirements_fulfillment: RequirementsFulfillment::empty(),
                            })));
                            let replacement_id = arguments[*idx];
                            truncate_tree(arguments.clone(), HashSet::from([replacement_id]), &mut implementation.expression_forest);
                            implementation.expression_forest.arguments.insert(expression_id, vec![replacement_id]);
                        }
                        InlineHint::YieldParameter(idx) => {
                            let replacement_id = arguments[*idx];
                            let new_operation = implementation.expression_forest.operations.remove(&replacement_id).unwrap();
                            let new_arguments = implementation.expression_forest.arguments.remove(&replacement_id).unwrap();
                            implementation.expression_forest.operations.insert(expression_id, new_operation);
                            implementation.expression_forest.arguments.insert(expression_id, new_arguments);
                            // We technically don't need to exclude replacement_id, but it's already been removed and would cause an error.
                            truncate_tree(arguments.clone(), HashSet::from([replacement_id]), &mut implementation.expression_forest);

                            // FIXME This is kind of lazy (recursive), but we may need to inline the expression again. Maybe we can unroll this into the loop somehow?
                            self.inline_expression(implementation, expression_id);
                        },
                        InlineHint::GlobalLookup(v) => {
                            implementation.expression_forest.operations.insert(expression_id, ExpressionOperation::VariableLookup(Rc::clone(v)));
                            // No matter what was passed before, we need no further arguments.
                            truncate_tree(arguments.clone(), HashSet::new(), &mut implementation.expression_forest);
                            implementation.expression_forest.arguments.insert(expression_id, vec![]);
                        },
                        InlineHint::NoOp => {
                            implementation.expression_forest.operations.remove(&expression_id);
                            truncate_tree(arguments.clone(), HashSet::new(), &mut implementation.expression_forest);
                        },
                    }
                }
            }
            ExpressionOperation::PairwiseOperations { .. } => {
                todo!()
            }
            _ => {}
        }
    }

    pub fn drain_all_functions_yield_uninlined(&mut self) -> Vec<Box<FunctionImplementation>> {
        self.implementation_by_head.retain(|head, _| !self.inline_hints.contains_key(head));
        self.implementation_by_head.drain().map(|(_, imp)| imp).collect()
    }
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

            guard!(let [arg] = &implementation.expression_forest.arguments[expression_id][..] else {
                return None
            });

            match &implementation.expression_forest.operations[arg] {
                ExpressionOperation::VariableLookup(v) => {
                    if let Some(idx) = implementation.parameter_variables.iter().position(|ref_| ref_ == v) {
                        return Some(InlineHint::ReplaceCall(Rc::clone(&f.function), idx))
                    }
                }
                _ => { },
            }

            return None
        },
        ExpressionOperation::VariableLookup(v) => {
            if let Some(idx) = implementation.parameter_variables.iter().position(|ref_| ref_ == v) {
                return Some(InlineHint::YieldParameter(idx))
            }
            else {
                return Some(InlineHint::GlobalLookup(Rc::clone(v)))
            }
        }
        _ => {},
    }

    None
}

pub fn truncate_tree(mut include: Vec<ExpressionID>, exclude: HashSet<ExpressionID>, forest: &mut ExpressionTree) {
    while let Some(next) = include.pop() {
        if exclude.contains(&next) {
            continue;
        }

        // It's enough to remove arguments and operations.
        // The type forest may still contain orphans, but that's ok. And our type doesn't change
        //  from inlining.
        forest.operations.remove(&next);
        include.extend(forest.arguments.remove(&next).unwrap());
    }
}
