pub mod constant_folding;
pub mod monomorphize;
pub mod inline;

use std::collections::{HashMap, HashSet};
use std::ops::DerefMut;
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use crate::interpreter::Runtime;
use crate::linker::interface::FunctionHead;
use crate::program::calls::FunctionBinding;
use crate::program::computation_tree::{ExpressionOperation, truncate_tree};
use crate::program::global::FunctionImplementation;
use crate::program::traits::RequirementsFulfillment;
use crate::refactor::inline::{inline_calls, try_inline};
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

        self.inline_hints.insert(Rc::clone(head), hint);

        for caller in self.callers.get(head).iter().flat_map(|x| x.iter()).cloned().collect_vec() {
            self.inline_calls(&caller);
        }
        self.invented_functions.retain(|f| f != head);

        return true
    }

    pub fn inline_calls(&mut self, head: &Rc<FunctionHead>) {
        let implementation = self.implementation_by_head.get_mut(head).unwrap();

        inline_calls(implementation, &self.inline_hints);

        self.update_callees(head);
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

