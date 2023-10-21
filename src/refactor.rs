pub mod simplify;
pub mod monomorphize;
pub mod inline;
pub mod locals;
pub mod analyze;
pub mod call_graph;

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::ops::DerefMut;
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use crate::interpreter::Runtime;
use crate::program::allocation::ObjectReference;
use crate::program::calls::FunctionBinding;
use crate::program::function_object::FunctionRepresentation;
use crate::program::functions::{FunctionHead, FunctionType};
use crate::program::global::{FunctionImplementation, FunctionLogic};
use crate::program::traits::RequirementsFulfillment;
use crate::refactor::call_graph::CallGraph;
use crate::refactor::inline::{inline_calls, InlineHint, try_inline};
use crate::refactor::monomorphize::Monomorphize;


pub struct Refactor<'a> {
    pub runtime: &'a mut Runtime,

    pub explicit_functions: Vec<Rc<FunctionHead>>,
    pub invented_functions: HashSet<Rc<FunctionHead>>,

    pub fn_representations: HashMap<Rc<FunctionHead>, FunctionRepresentation>,
    pub fn_logic: HashMap<Rc<FunctionHead>, FunctionLogic>,
    pub fn_inline_hints: HashMap<Rc<FunctionHead>, InlineHint>,

    pub call_graph: CallGraph,

    pub monomorphize: Monomorphize,
}

impl<'a> Refactor<'a> {
    pub fn new(runtime: &'a mut Runtime) -> Refactor<'a> {
        Refactor {
            runtime,
            explicit_functions: vec![],
            invented_functions: HashSet::new(),
            fn_representations: Default::default(),
            fn_logic: Default::default(),
            fn_inline_hints: Default::default(),
            call_graph: CallGraph::new(),
            monomorphize: Monomorphize::new(),
        }
    }

    pub fn add(&mut self, mut implementation: Box<FunctionImplementation>, representation: FunctionRepresentation) {
        self.explicit_functions.push(Rc::clone(&implementation.head));
        self._add(implementation, representation)
    }

    fn _add(&mut self, mut implementation: Box<FunctionImplementation>, representation: FunctionRepresentation) {
        let head = Rc::clone(&implementation.head);

        self.fn_logic.insert(Rc::clone(&head), FunctionLogic::Implementation(implementation));
        self.fn_representations.insert(Rc::clone(&head), representation);
        self.update_callees(&head);

        // New function; it may call functions that were already inlined!
        self.inline_calls(&head);
    }

    pub fn update_callees(&mut self, head: &Rc<FunctionHead>) {
        match &self.fn_logic[head] {
            FunctionLogic::Implementation(i) => {
                self.call_graph.change_callees(head, analyze::gather_callees(i))
            },
            FunctionLogic::Descriptor(_) => {},  // For now, descriptors are disallowed from calling monoteny functions anyway
        }
    }

    pub fn try_inline(&mut self, head: &Rc<FunctionHead>) -> Result<HashSet<Rc<FunctionHead>>, ()> {
        if self.explicit_functions.contains(head) {
            return Err(())
        }

        match self.fn_logic.entry(Rc::clone(head)) {
            Entry::Occupied(o) => {
                guard!(let FunctionLogic::Implementation(imp) = o.get() else {
                    return Err(())
                });
                guard!(let Some(inline) = try_inline(imp) else {
                    return Err(())
                });

                o.remove();
                self.fn_inline_hints.insert(Rc::clone(head), inline);
            }
            Entry::Vacant(_) => panic!(),
        }

        return Ok(self.apply_inline(head))
    }

    pub fn apply_inline(&mut self, head: &Rc<FunctionHead>) -> HashSet<Rc<FunctionHead>> {
        let affected = self.call_graph.callers.get(head).cloned().unwrap_or(HashSet::new());
        for caller in affected.iter() {
            self.inline_calls(caller);
        }

        affected
    }

    pub fn inline_calls(&mut self, head: &Rc<FunctionHead>) {
        if self.fn_inline_hints.is_empty() {
            return
        }

        match self.fn_logic.get_mut(head).unwrap() {
            FunctionLogic::Implementation(imp) => {
                inline_calls(imp, &self.fn_inline_hints);
                self.update_callees(head);
            }
            _ => {}
        }
    }

    pub fn monomorphize_calls(&mut self, head: Rc<FunctionHead>, should_monomorphize: &impl Fn(&Rc<FunctionBinding>) -> bool) -> HashSet<Rc<FunctionHead>> {
        guard!(let Some(FunctionLogic::Implementation(implementation)) = self.fn_logic.get_mut(&head) else {
            panic!();
        });

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
        self.update_callees(&head);

        while let Some(function_binding) = self.monomorphize.new_encountered_calls.pop() {
            // We may not create a new one through monomorphization, but we still need to take ownership.
            let logic = self.runtime.source.fn_logic.get(&function_binding.function).unwrap().clone();
            let representation = self.runtime.source.fn_representations[&function_binding.function].clone();

            match logic {
                FunctionLogic::Implementation(mut imp) => {
                    // If the call had an empty fulfillment, it wasn't monomorphized. We can just use the implementation itself!
                    if self.monomorphize.resolved_call_to_mono_call.contains_key(&function_binding) {
                        self.monomorphize.monomorphize_function(
                            &mut imp,
                            &function_binding,
                            should_monomorphize
                        );
                    };

                    new_heads.insert(Rc::clone(&imp.head));
                    self.invented_functions.insert(Rc::clone(&imp.head));
                    self._add(imp, representation);
                }
                FunctionLogic::Descriptor(d) => {
                    // It's internal logic! Can't monomorphize but we should still grab it.
                    self.fn_logic.insert(Rc::clone(&function_binding.function), FunctionLogic::Descriptor(d));
                    self.fn_representations.insert(Rc::clone(&function_binding.function), representation);
                }
            }
        }

        new_heads
    }

    pub fn remove_locals(&mut self, function: &Rc<FunctionHead>, removed_locals: &HashSet<Rc<ObjectReference>>) -> HashSet<Rc<FunctionHead>> {
        assert!(function.function_type == FunctionType::Static);

        guard!(let Some(FunctionLogic::Implementation(mut implementation)) = self.fn_logic.remove(function) else {
            panic!();
        });
        let changes_interface = removed_locals.iter().any(|l| implementation.parameter_locals.contains(l));
        if !changes_interface {
            // We can just change the function in-place!
            let param_swizzle = locals::remove_locals(&mut implementation, removed_locals);
            assert!(param_swizzle.is_none());
            self.fn_logic.insert(Rc::clone(function), FunctionLogic::Implementation(implementation));
            self.update_callees(function);
            // We changed the function; it is dirty!
            return HashSet::from([Rc::clone(function)])
        }

        // We need to create a new function; the interface changes and thus does the FunctionHead.

        let param_swizzle = locals::remove_locals(&mut implementation, removed_locals).unwrap();
        let new_head = Rc::clone(&implementation.head);

        self.invented_functions.insert(Rc::clone(&new_head));
        self.fn_inline_hints.insert(Rc::clone(function), InlineHint::ReplaceCall(Rc::clone(&new_head), param_swizzle.clone()));
        self.fn_logic.insert(Rc::clone(&new_head), FunctionLogic::Implementation(implementation));

        self.call_graph.callers.remove(function);
        self.call_graph.callees.remove(function);
        self.update_callees(&new_head);

        // The new function is also dirty!
        self.apply_inline(function).into_iter().chain([new_head].into_iter()).collect()
    }
}

