use std::collections::hash_map::RandomState;
use linked_hash_set::LinkedHashSet;
use crate::refactor::{Refactor, locals};
use crate::transpiler::Config;

pub struct Simplify<'a, 'b> {
    pub refactor: &'a mut Refactor<'b>,
    pub inline: bool,
    pub trim_locals: bool,
}

/// For now, the constant folder is only capable of inlining trivial functions:
/// - those that return a parameter
/// - those that call another function with one argument
/// - those that lookup a global variable (eg function reference)
/// - those that do nothing
///
impl<'a, 'b> Simplify<'a, 'b> {
    pub fn new(refactor: &'a mut Refactor<'b>, config: &Config) -> Simplify<'a, 'b> {
        Simplify {
            refactor,
            inline: config.should_inline,
            trim_locals: config.should_trim_locals,
        }
    }

    pub fn run(&mut self) {
        let mut next: LinkedHashSet<_, RandomState> = LinkedHashSet::from_iter(self.refactor.implementation_by_head.keys().cloned());
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
                let implementation = &self.refactor.implementation_by_head[&current];
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
                    let new_head = self.refactor.remove_locals(&current, &remove);

                    if self.inline {
                        self.refactor.inline_calls(&new_head);
                        let set = self.refactor.apply_inline(&current);
                        next.extend(set);
                    }
                }
            }
        }
    }
}
