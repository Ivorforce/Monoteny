use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use linked_hash_set::LinkedHashSet;
use crate::util::multimap::insert_into_multimap;

pub struct ChangeGraph<I: Eq + Hash + Clone> {
    pub next: LinkedHashSet<I>,
    pub dependents: HashMap<I, HashSet<I>>,
}

impl<I: Eq + Hash + Clone> ChangeGraph<I> {
    pub fn new() -> ChangeGraph<I> {
        ChangeGraph {
            next: LinkedHashSet::new(),
            dependents: Default::default(),
        }
    }

    pub fn pop(&mut self) -> Option<I> {
        self.next.pop_front()
    }

    pub fn mark_change(&mut self, object: I) {
        if self.next.insert(object.clone()) {
            // Could use a LinkedHashSet here too, but that's not really worth it speed wise.
            let mut next = vec![object];

            while let Some(object) = next.pop() {
                if self.next.insert(object.clone()) {
                    next.push(object);
                }
            }
        }
    }

    pub fn remove_dependency(&mut self, dependent: &I, dependency: &I) {
        self.dependents.get_mut(dependency).map(|s| s.remove(dependent));
    }

    pub fn add_dependency(&mut self, dependent: I, dependency: I) {
        insert_into_multimap(&mut self.dependents, dependent, dependency);
    }

    pub fn add_dependencies(&mut self, dependent: &I, dependencies: impl Iterator<Item=I>) {
        for dependency in dependencies {
            self.add_dependency(dependent.clone(), dependency)
        }
    }
}
