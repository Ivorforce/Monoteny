use linked_hash_set::LinkedHashSet;
use std::collections::hash_map::RandomState;
use std::rc::Rc;

use crate::program::functions::{FunctionHead, FunctionLogic};
use crate::refactor::{locals, Refactor};
use crate::source::Source;

pub struct Simplify {
    pub refactor: Refactor,
    pub inline: bool,
    pub trim_locals: bool,
    pub monomorphize: bool,
}

impl Simplify {
    pub fn run<'a>(&mut self, from_functions: impl Iterator<Item=&'a Rc<FunctionHead>>, source: &Source) {
        if self.monomorphize {
            // First, monomorphize everything we call
            let mut next: LinkedHashSet<_, RandomState> = LinkedHashSet::from_iter(
                self.refactor.explicit_functions.iter()
                    .flat_map(|head| self.refactor.call_graph.callees[head].iter().cloned())
            );
            while let Some(current) = next.pop_front() {
                if let Some(monomorphized) = self.refactor.try_monomorphize(&current, source) {
                    next.extend(self.refactor.call_graph.callees.get(&monomorphized).unwrap().iter().cloned());
                }
            }
        }

        // Make sure refactor has everything that's needed so we can simplify it.
        self.refactor.gather_deep_functions(from_functions, source);

        // Now, let's simplify!
        let mut next: LinkedHashSet<_, RandomState> = LinkedHashSet::from_iter(self.refactor.fn_logic.keys().cloned());
        while let Some(current) = next.pop_front() {
            // TODO The explicit functions should be refactorable too, I think.
            let is_explicit = self.refactor.explicit_functions.contains(&current);

            if !is_explicit && self.inline {
                // Try to inline the function if it's trivial.
                if let Ok(affected) = self.refactor.try_inline(&current) {
                    // Try inlining those that changed again.
                    // TODO This could be more efficient: It only makes sense to change functions once.
                    //  The inlining call can be delayed until we're sure we can either be inlined
                    //  ourselves, or we just postpone it until everything else is done.
                    next.extend(affected);

                    // The function was inlined; there's no need to do anything else.
                    continue
                }
            }

            // Try to remove unused parameters for the function.
            if self.trim_locals {
                if let FunctionLogic::Implementation(implementation) = &self.refactor.fn_logic[&current] {
                    // TODO What if the parameters' setters call I/O functions?
                    //  We should only remove those that aren't involved in I/O. We can actually
                    //  remove any as long as they're not involved in I/O.
                    let mut remove = locals::find_unused_locals(implementation);

                    if is_explicit {
                        // TODO Cannot change interface for now because it replaces the function head,
                        //  which may be in use elsewhere.
                        implementation.parameter_locals.iter().for_each(|l| _ = remove.remove(l));
                    }

                    if !remove.is_empty() {
                        next.extend(self.refactor.swizzle_implementation(&current, |imp| {
                            locals::remove_locals(imp, &remove)
                        }));
                    }
                };
            }
        }
    }
}
