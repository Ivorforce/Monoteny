use std::collections::{HashMap, HashSet};
use std::ops::Index;
use std::os::macos::raw::stat;
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use crate::program::allocation::ObjectReference;
use crate::program::calls::FunctionBinding;
use crate::program::computation_tree::{ExpressionForest, ExpressionID, ExpressionOperation, Statement};
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
            guard!(let Some(operation) = implementation.expression_forest.operations.get(&expression_id) else {
                continue;
            });
            let arguments = &implementation.expression_forest.arguments[&expression_id].clone();

            match operation {
                ExpressionOperation::FunctionCall(f) => {
                    if let Some(inline_hint) = self.inline_hints.get(&f.function) {
                        match inline_hint {
                            InlineHint::ReplaceCall(target_function, idx) => {
                                implementation.expression_forest.operations.insert(expression_id, ExpressionOperation::FunctionCall(Rc::new(FunctionBinding {
                                    function: Rc::clone(&target_function),
                                    requirements_fulfillment: RequirementsFulfillment::empty(),
                                })));
                                let replacement_id = arguments[*idx];
                                truncate_tree(arguments.clone(), HashSet::from([replacement_id]), &mut implementation.expression_forest);
                                implementation.expression_forest.arguments.insert(expression_id, vec![replacement_id]);
                            }
                            InlineHint::YieldParameter(idx) => {
                                let replacement_id = arguments[*idx];
                                let new_operation = implementation.expression_forest.operations.remove(&replacement_id).unwrap();
                                implementation.expression_forest.operations.insert(expression_id, new_operation);
                                truncate_tree(arguments.clone(), HashSet::from([replacement_id]), &mut implementation.expression_forest);
                                implementation.expression_forest.arguments.insert(expression_id, vec![replacement_id]);
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

        // Reverse iteration allows us to remove objects without indexes getting invalidated.
        for i in (0 .. implementation.statements.len()).rev() {
            match &implementation.statements[i].as_ref() {
                Statement::VariableAssignment(v, e) => {
                    if !implementation.expression_forest.operations.contains_key(e) {
                        panic!("Accidentally truncated variable assignment.")
                    }
                },
                Statement::Expression(e) => {
                    if !implementation.expression_forest.operations.contains_key(e) {
                        implementation.statements.remove(i);
                    }
                }
                Statement::Return(Some(e)) => {
                    if !implementation.expression_forest.operations.contains_key(e) {
                        panic!("Accidentally truncated return statement.")
                    }
                }
                Statement::Return(None) => {},
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

    if let [statement] = &implementation.statements[..] {
        match statement.as_ref() {
            Statement::Expression(e) => get_trivial_expression_call_target(e, implementation),
            Statement::Return(Some(e)) => get_trivial_expression_call_target(e, implementation),
            Statement::Return(None) => Some(InlineHint::NoOp),
            _ => None,
        }
    }
    else {
        None
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

pub fn truncate_tree(mut include: Vec<ExpressionID>, exclude: HashSet<ExpressionID>, forest: &mut ExpressionForest) {
    while let Some(next) = include.pop() {
        if exclude.contains(&next) {
            continue;
        }

        forest.operations.remove(&next);
        include.extend(forest.arguments.remove(&next).unwrap());
    }
}
