pub mod constant_folding;
pub mod monomorphize;

use std::collections::{HashMap, HashSet};
use std::ops::DerefMut;
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use crate::interpreter::Runtime;
use crate::linker::interface::FunctionHead;
use crate::program::calls::FunctionBinding;
use crate::program::computation_tree::{ExpressionID, ExpressionOperation, truncate_tree};
use crate::program::global::FunctionImplementation;
use crate::program::traits::RequirementsFulfillment;
use crate::refactor::monomorphize::Monomorphize;
use crate::util::iter::omega;
use crate::util::multimap::{insert_into_multimap, remove_from_multimap};


#[derive(Clone, Debug)]
pub enum InlineHint {
    ReplaceCall(Rc<FunctionHead>, Vec<usize>),
    YieldParameter(usize),
    NoOp,
}

pub struct Refactor<'a> {
    pub runtime: &'a Runtime,

    pub explicit_functions: Vec<Rc<FunctionHead>>,
    pub invented_functions: Vec<Rc<FunctionHead>>,

    pub implementation_by_head: HashMap<Rc<FunctionHead>, Box<FunctionImplementation>>,
    pub callers: HashMap<Rc<FunctionHead>, HashSet<Rc<FunctionHead>>>,
    pub callees: HashMap<Rc<FunctionHead>, HashSet<Rc<FunctionHead>>>,

    pub inline_hints: HashMap<Rc<FunctionHead>, InlineHint>,

    pub monomorphize: Monomorphize,
}

impl<'a> Refactor<'a> {
    pub fn new(runtime: &'a Runtime) -> Refactor<'a> {
        Refactor {
            runtime,
            implementation_by_head: Default::default(),
            explicit_functions: vec![],
            invented_functions: vec![],
            callers: Default::default(),
            callees: Default::default(),
            inline_hints: Default::default(),
            monomorphize: Monomorphize::new(),
        }
    }

    pub fn add(&mut self, mut implementation: Box<FunctionImplementation>) {
        self.explicit_functions.push(Rc::clone(&implementation.head));
        self._add(implementation)
    }

    fn _add(&mut self, mut implementation: Box<FunctionImplementation>) {
        let head = Rc::clone(&implementation.head);

        self.implementation_by_head.insert(Rc::clone(&head), implementation);
        self.update_callees(&head);

        if !self.inline_hints.is_empty() {
            // In case any calls have already been inlined before this implementation was added.
            self.inline_calls(&head);
        }
    }

    pub fn update_callees(&mut self, head: &Rc<FunctionHead>) {
        if let Some(previous_callees) = self.callees.get(head) {
            for previous_callee in previous_callees.iter() {
                remove_from_multimap(&mut self.callers, previous_callee, head);
            }
        }
        let new_callees = gather_callees(&self.implementation_by_head[head]);
        for callee in new_callees.iter() {
            insert_into_multimap(&mut self.callers, Rc::clone(callee), Rc::clone(head));
        }
        self.callees.insert(Rc::clone(head), new_callees);
    }

    pub fn try_inline(&mut self, head: &Rc<FunctionHead>) -> bool {
        if self.explicit_functions.contains(head) {
            return false
        }

        guard!(let Some(imp) = self.implementation_by_head.get_mut(head) else {
            return false
        });

        guard!(let Some(hint) = try_inline(imp) else {
            return false
        });

        self._inline_cascade(head, hint);

        for caller in self.callers.get(head).iter().flat_map(|x| x.iter()).cloned().collect_vec() {
            self.inline_calls(&caller);
        }
        self.invented_functions.retain(|f| f != head);

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

        self.update_callees(head);
    }

    fn _inline_cascade(&mut self, head: &Rc<FunctionHead>, hint: InlineHint) {
        let all_affected = omega([(head, hint)].into_iter(), |(head, hint)| {
            return self.callers.get(*head).iter().flat_map(|x| x.iter())
                .filter_map(|caller| self.inline_hints.remove(caller).map(|hint| (caller, hint)))
                .collect_vec().into_iter()
        }).collect_vec();

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

    pub fn monomorphize(&mut self, head: Rc<FunctionHead>, should_monomorphize: &impl Fn(&Rc<FunctionBinding>) -> bool) {
        let mut implementation = self.implementation_by_head.get_mut(&head).unwrap();

        if !implementation.head.interface.generics.is_empty() {
            // We'll need to somehow transpile requirements as protocols and generics as generics.
            // That's for later!
            panic!("Transpiling generic functions is not supported yet: {:?}", implementation.head);
        }

        self.monomorphize.monomorphize_function(
            implementation,
            &Rc::new(FunctionBinding {
                // The implementation's pointer is fine.
                function: Rc::clone(&head),
                // The resolution SHOULD be empty: The function is transpiled WITH its generics.
                // Unless generics are bound in the transpile directive, which is TODO
                requirements_fulfillment: RequirementsFulfillment::empty(),
            }),
            should_monomorphize
        );
        self.update_callees(&head);

        while let Some(function_binding) = self.monomorphize.new_encountered_calls.pop() {
            guard!(let Some(implementation) = self.runtime.source.fn_implementations.get(&function_binding.function) else {
            // We don't have an implementation ready, so it must be a core or otherwise injected.
            continue;
        });

            // We may not create a new one through monomorphization, but we still need to take ownership.
            let mut mono_implementation = implementation.clone();
            // If the call had an empty fulfillment, it wasn't monomorphized. We can just use the implementation itself!
            if self.monomorphize.resolved_call_to_mono_call.contains_key(&function_binding) {
                self.monomorphize.monomorphize_function(
                    &mut mono_implementation,
                    &function_binding,
                    should_monomorphize
                );
            };

            self.invented_functions.push(Rc::clone(&mono_implementation.head));
            self._add(mono_implementation);
        }
    }
}

pub fn gather_callees(implementation: &FunctionImplementation) -> HashSet<Rc<FunctionHead>> {
    let mut callees = HashSet::new();

    for expression_op in implementation.expression_forest.operations.values() {
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

