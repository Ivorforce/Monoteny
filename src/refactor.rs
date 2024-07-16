use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::rc::Rc;

use itertools::Itertools;
use linked_hash_set::LinkedHashSet;

use crate::interpreter::runtime::Runtime;
use crate::program::functions::{FunctionBinding, FunctionHead, FunctionImplementation, FunctionLogic, FunctionLogicDescriptor, FunctionType};
use crate::refactor::call_graph::CallGraph;
use crate::refactor::inline::{inline_calls, InlineHint, try_inline};
use crate::refactor::monomorphize::monomorphize_implementation;

pub mod simplify;
pub mod monomorphize;
pub mod inline;
pub mod locals;
pub mod analyze;
pub mod call_graph;

pub struct Refactor {
    pub explicit_functions: Vec<Rc<FunctionHead>>,
    pub invented_functions: HashSet<Rc<FunctionHead>>,

    pub fn_logic: HashMap<Rc<FunctionHead>, FunctionLogic>,
    pub fn_inline_hints: HashMap<Rc<FunctionHead>, InlineHint>,
    pub fn_optimizations: HashMap<Rc<FunctionBinding>, Rc<FunctionHead>>,

    pub call_graph: CallGraph,
}

impl Refactor {
    pub fn new() -> Refactor {
        Refactor {
            explicit_functions: vec![],
            invented_functions: HashSet::new(),
            fn_logic: Default::default(),
            fn_inline_hints: Default::default(),
            fn_optimizations: Default::default(),
            call_graph: CallGraph::new(),
        }
    }

    pub fn add(&mut self, mut implementation: Box<FunctionImplementation>) {
        self.explicit_functions.push(Rc::clone(&implementation.head));
        self._add(implementation)
    }

    fn _add(&mut self, mut implementation: Box<FunctionImplementation>) {
        let head = Rc::clone(&implementation.head);

        self.fn_logic.insert(Rc::clone(&head), FunctionLogic::Implementation(implementation));
        self.update_callees(&head);

        // New function; it may call functions that were already inlined!
        self.inline_calls_from(&head);
    }

    /// Place or replace a function with a stub.
    ///  This can be useful to 'mark' the function as the compiler intending to inject it itself.
    pub fn replace_with_stub(&mut self, head: &Rc<FunctionHead>) {
        self.call_graph.change_callees(head, LinkedHashSet::new());
        self.fn_logic.insert(Rc::clone(head), FunctionLogic::Descriptor(FunctionLogicDescriptor::Stub));
    }

    pub fn update_callees(&mut self, head: &Rc<FunctionHead>) {
        match &self.fn_logic[head] {
            FunctionLogic::Implementation(i) => {
                self.call_graph.change_callees(head, analyze::gather_callees(i))
            },
            FunctionLogic::Descriptor(_) => {
                // For now, descriptors are disallowed from calling monoteny functions anyway
                self.call_graph.change_callees(head, LinkedHashSet::new())
            },
        }
    }

    pub fn try_inline(&mut self, head: &Rc<FunctionHead>) -> Result<HashSet<Rc<FunctionHead>>, ()> {
        if self.explicit_functions.contains(head) {
            return Err(())
        }

        let Entry::Occupied(o) = self.fn_logic.entry(Rc::clone(head)) else {
            panic!("(Internal Error) Tried to inline an unknown function: {:?}", head);
        };

        let FunctionLogic::Implementation(imp) = o.get() else {
            return Err(())
        };

        let Some(inline) = try_inline(imp) else {
            return Err(())
        };

        o.remove();
        self.fn_inline_hints.insert(Rc::clone(head), inline);

        return Ok(self.inline_calls_to(head))
    }

    pub fn inline_calls_to(&mut self, head: &Rc<FunctionHead>) -> HashSet<Rc<FunctionHead>> {
        let affected: HashSet<_> = self.call_graph.get_callers(head).cloned().collect();
        for caller in affected.iter() {
            self.inline_calls_from(caller);
        }
        self.call_graph.remove(head);

        affected
    }

    pub fn inline_calls_from(&mut self, head: &Rc<FunctionHead>) {
        match self.fn_logic.get_mut(head).unwrap() {
            FunctionLogic::Implementation(imp) => {
                inline_calls(imp, &self.fn_optimizations, &self.fn_inline_hints);
                self.update_callees(head);
            }
            // 'Native' functions can't be inlined.
            FunctionLogic::Descriptor(_) => {}
        }
    }

    pub fn try_monomorphize(&mut self, binding: &Rc<FunctionBinding>, runtime: &mut Runtime) -> Option<Rc<FunctionHead>> {
        if self.fn_optimizations.contains_key(binding) {
            return None  // We already have an optimization; we need not monomorphize.
        }

        let Some(logic) = self.fn_logic.get(&binding.function).or_else(|| runtime.source.fn_logic.get(&binding.function)) else {
            panic!("Cannot find logic for function {:?}", binding.function);
        };

        let FunctionLogic::Implementation(implementation) = logic else {
            return None
        };

        let mut new_implementation = implementation.clone();
        monomorphize_implementation(&mut new_implementation, binding);
        let mono_head = Rc::clone(&new_implementation.head);

        self.fn_optimizations.insert(Rc::clone(binding), Rc::clone(&mono_head));

        self.fn_logic.insert(Rc::clone(&mono_head), FunctionLogic::Implementation(new_implementation));

        // Set the initial callees (none if it's a stub)
        self.update_callees(&mono_head);
        // After monomorphizing, we may call functions that have been inlined already.
        // Let's change that now!
        self.inline_calls_from(&mono_head);

        // Find all callers we can optimize and make them call us instead.
        for caller in self.call_graph.get_binding_callers(binding).cloned().collect_vec() {
            self.inline_calls_from(&caller);
        }

        Some(mono_head)
    }

    /// Map an implementation. If the implementation's head is changed, the mapper must provide an inline hint.
    pub fn swizzle_implementation(&mut self, function: &Rc<FunctionHead>, map: impl Fn(&mut FunctionImplementation) -> Option<Vec<usize>>) -> HashSet<Rc<FunctionHead>> {
        assert!(function.function_type == FunctionType::Static);

        let Some(FunctionLogic::Implementation(mut implementation)) = self.fn_logic.remove(function) else {
            panic!();
        };

        if let Some(swizzle) = map(&mut implementation) {
            // The mapper changed the interface / function ID!
            assert_ne!(function, &implementation.head);
            let new_head = Rc::clone(&implementation.head);

            self.invented_functions.insert(Rc::clone(&new_head));
            self.fn_inline_hints.insert(Rc::clone(function), InlineHint::ReplaceCall(Rc::clone(&implementation.head), swizzle));
            self.fn_logic.insert(Rc::clone(&new_head), FunctionLogic::Implementation(implementation));

            // Find the initial callees.
            self.update_callees(&new_head);

            // Gather everyone that's dirty! That includes the new function!
            let changed_functions = self.inline_calls_to(function).into_iter().chain([new_head].into_iter()).collect();

            // Remove the last function from the call graph. After apply_inline has been called,
            // we don't need this anymore, since we're inlined.
            self.call_graph.remove(function);

            changed_functions
        }
        else {
            // The function kept its interface!
            assert_eq!(function, &implementation.head);

            self.fn_logic.insert(Rc::clone(function), FunctionLogic::Implementation(implementation));
            self.update_callees(function);
            // We changed the function; it is dirty!
            return HashSet::from([Rc::clone(function)])
        }
    }

    pub fn gather_needed_functions(&mut self, runtime: &mut Runtime) -> LinkedHashSet<Rc<FunctionHead>> {
        let callees = self.call_graph.deep_callees(self.explicit_functions.iter());
        for callee in callees.iter() {
            if !self.fn_logic.contains_key(callee) {
                self.fn_logic.insert(Rc::clone(callee), runtime.source.fn_logic[callee].clone());
            }
        }
        callees
    }
}

