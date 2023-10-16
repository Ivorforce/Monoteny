use std::collections::hash_map::RandomState;
use linked_hash_set::LinkedHashSet;
use crate::refactor::Refactor;

pub struct ConstantFold<'a, 'b> {
    pub refactor: &'a mut Refactor<'b>,
}

/// For now, the constant folder is only capable of inlining trivial functions:
/// - those that return a parameter
/// - those that call another function with one argument
/// - those that lookup a global variable (eg function reference)
/// - those that do nothing
///
impl<'a, 'b> ConstantFold<'a, 'b> {
    pub fn new(refactor: &'a mut Refactor<'b>) -> ConstantFold<'a, 'b> {
        ConstantFold {
            refactor,
        }
    }

    pub fn run(&mut self) {
        let mut next: LinkedHashSet<_, RandomState> = LinkedHashSet::from_iter(self.refactor.invented_functions.iter().cloned());
        while let Some(current) = next.pop_front() {
            if self.refactor.try_inline(&current) {
                if let Some(dependents) = self.refactor.callers.get(&current) {
                    // Try inlining those that changed again.
                    // TODO This could be more efficient: It only makes sense to change functions once.
                    //  The inlining call can be delayed until we're sure we can either be inlined
                    //  ourselves, or we just postpone it until everything else is done.
                    next.extend(dependents.iter().cloned())
                }
            }
        }
    }
}
