use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub struct TaskDependencyGraph<I: Eq + Hash + Clone> {
    pub done: HashSet<I>,
    pub next: Vec<I>,
    pub changed: HashSet<I>,
    pub dependencies: HashMap<I, HashSet<I>>,
    pub dependents: HashMap<I, HashSet<I>>,
}

impl<I: Eq + Hash + Clone> TaskDependencyGraph<I> {
    pub fn new() -> TaskDependencyGraph<I> {
        TaskDependencyGraph {
            done: Default::default(),
            next: vec![],
            changed: Default::default(),
            dependencies: Default::default(),
            dependents: Default::default(),
        }
    }

    pub fn mark_as_done(&mut self, object: I) {
        let mut next = vec![object];

        while let Some(object) = next.pop() {
            self.dependencies.remove(&object);
            self.changed.remove(&object);

            for dependent in self.dependents.remove(&object).into_iter().flatten() {
                let dependencies = self.dependencies.get_mut(&dependent).unwrap();
                dependencies.remove(&object);
                if dependencies.is_empty() {
                    self.changed.insert(dependent.clone());
                    next.push(dependent);
                }
            }

            self.done.insert(object);
        }
    }

    pub fn add_task(&mut self, object: I, dependencies: HashSet<I>) {
        let remaining: HashSet<I> = dependencies.difference(&self.done).cloned().collect();

        match remaining.is_empty() {
            true => {
                self.next.push(object.clone());
                self.mark_as_done(object);
            }
            false => {
                for dependency in remaining.iter() {
                    match self.dependents.entry(dependency.clone()) {
                        Entry::Occupied(mut o) => _ = o.get_mut().insert(object.clone()),
                        Entry::Vacant(v) => _ = v.insert(HashSet::from([object.clone()]))
                    }
                }
                if remaining.len() < dependencies.len() {
                    self.changed.insert(object.clone());
                }
                self.dependencies.insert(object, remaining);
            }
        }
    }
}
