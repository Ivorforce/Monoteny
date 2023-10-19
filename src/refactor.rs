pub mod simplify;
pub mod monomorphize;
pub mod inline;
pub mod locals;

use std::collections::{HashMap, HashSet};
use std::ops::DerefMut;
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use linked_hash_set::LinkedHashSet;
use crate::interpreter::Runtime;
use crate::linker::interface::FunctionHead;
use crate::program::allocation::ObjectReference;
use crate::program::calls::FunctionBinding;
use crate::program::computation_tree::ExpressionOperation;
use crate::program::functions::FunctionType;
use crate::program::global::FunctionImplementation;
use crate::program::traits::RequirementsFulfillment;
use crate::refactor::inline::{inline_calls, try_inline};
use crate::refactor::monomorphize::Monomorphize;
use crate::util::multimap::{insert_into_multimap, remove_from_multimap};


#[derive(Clone, Debug)]
pub enum InlineHint {
    ReplaceCall(Rc<FunctionHead>, Vec<usize>),
    YieldParameter(usize),
    NoOp,
}

pub struct Refactor<'a> {
    pub runtime: &'a mut Runtime,

    pub explicit_functions: Vec<Rc<FunctionHead>>,
    pub invented_functions: HashSet<Rc<FunctionHead>>,

    pub implementation_by_head: HashMap<Rc<FunctionHead>, Box<FunctionImplementation>>,
    pub callers: HashMap<Rc<FunctionHead>, HashSet<Rc<FunctionHead>>>,
    pub callees: HashMap<Rc<FunctionHead>, LinkedHashSet<Rc<FunctionHead>>>,

    pub inline_hints: HashMap<Rc<FunctionHead>, InlineHint>,

    pub monomorphize: Monomorphize,
}

impl<'a> Refactor<'a> {
    pub fn new(runtime: &'a mut Runtime) -> Refactor<'a> {
        Refactor {
            runtime,
            implementation_by_head: Default::default(),
            explicit_functions: vec![],
            invented_functions: HashSet::new(),
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

        // New function; it may call functions that were already inlined!
        self.inline_calls(&head);
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

    pub fn try_inline(&mut self, head: &Rc<FunctionHead>) -> Result<HashSet<Rc<FunctionHead>>, ()> {
        if self.explicit_functions.contains(head) {
            return Err(())
        }

        guard!(let Some(imp) = self.implementation_by_head.get_mut(head) else {
            return Err(())
        });

        guard!(let Some(hint) = try_inline(imp) else {
            return Err(())
        });

        self.inline_hints.insert(Rc::clone(head), hint);;

        return Ok(self.apply_inline(head))
    }

    pub fn apply_inline(&mut self, head: &Rc<FunctionHead>) -> HashSet<Rc<FunctionHead>> {
        let affected = self.callers.get(head).cloned().unwrap_or(HashSet::new());
        for caller in affected.iter() {
            self.inline_calls(caller);
        }

        affected
    }

    pub fn inline_calls(&mut self, head: &Rc<FunctionHead>) {
        if self.inline_hints.is_empty() {
            return
        }

        let implementation = self.implementation_by_head.get_mut(head).unwrap();

        inline_calls(implementation, &self.inline_hints);

        self.update_callees(head);
    }

    pub fn monomorphize(&mut self, head: Rc<FunctionHead>, should_monomorphize: &impl Fn(&Rc<FunctionBinding>) -> bool) -> HashSet<Rc<FunctionHead>> {
        let mut implementation = self.implementation_by_head.get_mut(&head).unwrap();
        let mut new_heads = HashSet::new();

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
        let new_head = Rc::clone(&implementation.head);
        new_heads.insert(Rc::clone(&new_head));
        self.runtime.source.fn_implementations.insert(Rc::clone(&new_head), implementation.clone());
        self.copy_quirks_source(&head, &new_head);
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

            new_heads.insert(Rc::clone(&mono_implementation.head));
            self.invented_functions.insert(Rc::clone(&mono_implementation.head));
            self.runtime.source.fn_implementations.insert(Rc::clone(&mono_implementation.head), mono_implementation.clone());
            self.copy_quirks_source(&function_binding.function, &mono_implementation.head);
            self._add(mono_implementation);
        }

        new_heads
    }

    pub fn remove_locals(&mut self, function: &Rc<FunctionHead>, removed_locals: &HashSet<Rc<ObjectReference>>) -> HashSet<Rc<FunctionHead>> {
        assert!(function.function_type == FunctionType::Static);

        let mut old_implementation = self.implementation_by_head.get_mut(function).unwrap();
        let changes_interface = removed_locals.iter().any(|l| old_implementation.parameter_locals.contains(l));
        if !changes_interface {
            // We can just change the function in-place!
            let param_swizzle = locals::remove_locals(old_implementation, removed_locals);
            assert!(param_swizzle.is_none());
            self.update_callees(function);
            // We changed the function; it is dirty!
            return HashSet::from([Rc::clone(function)])
        }

        // We need to create a new function; the interface changes and thus does the FunctionHead.

        let mut new_implementation = old_implementation.clone();
        let param_swizzle = locals::remove_locals(&mut new_implementation, removed_locals).unwrap();
        let new_head = Rc::clone(&new_implementation.head);

        self.invented_functions.insert(Rc::clone(&new_head));
        self.runtime.source.fn_implementations.insert(Rc::clone(&new_head), new_implementation.clone());
        self.implementation_by_head.insert(Rc::clone(&new_head), new_implementation);
        self.copy_quirks_source(function, &new_head);

        self.update_callees(&new_head);

        // We do not remove the old function. The old function CAN be inlined to call the new one, but it doesn't have to.
        // This is especially important if we get new callers to the old function later. All information is retained.
        self.inline_hints.insert(Rc::clone(function), InlineHint::ReplaceCall(Rc::clone(&new_head), param_swizzle.clone()));

        // The new function is also dirty!
        self.apply_inline(function).into_iter().chain([new_head].into_iter()).collect()
    }

    fn copy_quirks_source(&mut self, from: &Rc<FunctionHead>, to: &Rc<FunctionHead>) {
        self.runtime.source.fn_representations.get(from).cloned().map(|rep| self.runtime.source.fn_representations.insert(Rc::clone(to), rep));
        self.runtime.source.fn_builtin_hints.get(from).cloned().map(|hint| self.runtime.source.fn_builtin_hints.insert(Rc::clone(to), hint));
    }

    pub fn required_functions(&self) -> LinkedHashSet<Rc<FunctionHead>> {
        let mut next = self.explicit_functions.iter().collect_vec();
        let mut gathered = LinkedHashSet::new();
        while let Some(current) = next.pop() {
            assert!(!self.inline_hints.contains_key(current), "Internal error; forgot to inline a function call.");
            guard!(let Some(callees) = self.callees.get(current) else {
                continue  // Probably an internal function
            });
            gathered.extend(callees.iter().cloned());
            next.extend(callees.iter())
        }
        gathered
    }
}

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

