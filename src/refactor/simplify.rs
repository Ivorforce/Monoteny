use std::collections::hash_map::RandomState;
use linked_hash_set::LinkedHashSet;
use crate::program::global::FunctionLogic;
use crate::refactor::{Refactor, locals};
use crate::transpiler::Config;

pub struct Simplify<'a, 'b> {
    pub refactor: &'a mut Refactor<'b>,
    pub inline: bool,
    pub trim_locals: bool,
    pub monomorphize: bool,
}

impl<'a, 'b> Simplify<'a, 'b> {
    pub fn new(refactor: &'a mut Refactor<'b>, config: &Config) -> Simplify<'a, 'b> {
        if !config.should_monomorphize {
            todo!();  // Lots of reasons non-monomorphization doesn't work right now.
        }

        Simplify {
            refactor,
            inline: config.should_inline,
            trim_locals: config.should_trim_locals,
            monomorphize: config.should_monomorphize,
        }
    }

    pub fn run(&mut self) {
        if self.monomorphize {
            // First, monomorphize everything we call
            let mut next: LinkedHashSet<_, RandomState> = LinkedHashSet::from_iter(
                self.refactor.explicit_functions.iter()
                    .flat_map(|head| self.refactor.call_graph.callees[head].iter().cloned())
            );
            while let Some(current) = next.pop_front() {
                if let Some(monomorphized) = self.refactor.try_monomorphize(&current) {
                    next.extend(self.refactor.call_graph.callees.get(&monomorphized).unwrap().iter().cloned());
                }
            }
        }

        // Make sure refactor has everything that's needed so we can simplify it.
        self.refactor.gather_needed_functions();

        // Now, let's simplify!
        let mut next: LinkedHashSet<_, RandomState> = LinkedHashSet::from_iter(self.refactor.fn_logic.keys().cloned());
        while let Some(current) = next.pop_front() {
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
