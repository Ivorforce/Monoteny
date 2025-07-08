use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use itertools::Itertools;
use linked_hash_set::LinkedHashSet;

use crate::program::functions::{FunctionBinding, FunctionHead};
use crate::util::multimap::{insert_into_multimap, remove_from_multimap};

pub struct CallGraph {
    pub callers: HashMap<Rc<FunctionHead>, HashMap<Rc<FunctionBinding>, HashSet<Rc<FunctionHead>>>>,
    pub callees: HashMap<Rc<FunctionHead>, LinkedHashSet<Rc<FunctionBinding>>>,
}

impl CallGraph {
    pub fn new() -> CallGraph {
        CallGraph {
            callers: Default::default(),
            callees: Default::default(),
        }
    }

    pub fn get_callers(&self, head: &Rc<FunctionHead>) -> impl Iterator<Item=&Rc<FunctionHead>> {
        self.callers.get(head).into_iter()
            .flat_map(|cs| cs.values())
            .flatten()
            .dedup()
    }

    pub fn get_binding_callers<'a>(&'a self, binding: &'a Rc<FunctionBinding>) -> impl Iterator<Item=&'a Rc<FunctionHead>> {
        self.callers.get(&binding.function).into_iter()
            .flat_map(|c| c.get(binding).into_iter())
            .flatten()
    }

    pub fn remove(&mut self, head: &Rc<FunctionHead>) {
        self.clear_callees(head);
        self.callers.remove(head);
        self.callees.remove(head);
    }

    pub fn clear_callees(&mut self, head: &Rc<FunctionHead>) {
        if let Some(previous_callees) = self.callees.get(head) {
            for previous_callee in previous_callees.iter() {
                if let Entry::Occupied(mut o) = self.callers.entry(Rc::clone(&previous_callee.function)) {
                    remove_from_multimap(o.get_mut(), previous_callee, head);
                }
            }
        }
    }

    pub fn change_callees(&mut self, head: &Rc<FunctionHead>, new_callees: LinkedHashSet<Rc<FunctionBinding>>) {
        self.clear_callees(head);
        for callee_binding in new_callees.iter() {
            match self.callers.entry(Rc::clone(&callee_binding.function)) {
                Entry::Occupied(mut o) => {
                    insert_into_multimap(o.get_mut(), Rc::clone(callee_binding), Rc::clone(head));
                }
                Entry::Vacant(mut v) => {
                    v.insert(HashMap::from([(Rc::clone(callee_binding), HashSet::from([Rc::clone(head)]))]));
                }
            }
        }
        self.callees.insert(Rc::clone(head), new_callees);
    }

    pub fn deep_callees<'a>(&self, from: impl Iterator<Item=&'a Rc<FunctionHead>>) -> LinkedHashSet<Rc<FunctionHead>> {
        let mut next = from.collect_vec();
        let mut gathered = LinkedHashSet::new();
        while let Some(current) = next.pop() {
            let Some(callees) = self.callees.get(current) else {
                continue
            };
            gathered.extend(callees.iter().map(|f| &f.function).cloned());
            next.extend(callees.iter().map(|f| &f.function))
        }
        gathered
    }
}
