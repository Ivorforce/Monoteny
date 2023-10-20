use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use linked_hash_set::LinkedHashSet;
use crate::program::functions::FunctionHead;
use crate::util::multimap::{insert_into_multimap, remove_from_multimap};

pub struct CallGraph {
    pub callers: HashMap<Rc<FunctionHead>, HashSet<Rc<FunctionHead>>>,
    pub callees: HashMap<Rc<FunctionHead>, LinkedHashSet<Rc<FunctionHead>>>,
}

impl CallGraph {
    pub fn new() -> CallGraph {
        CallGraph {
            callers: Default::default(),
            callees: Default::default(),
        }
    }

    pub fn change_callees(&mut self, head: &Rc<FunctionHead>, new_callees: LinkedHashSet<Rc<FunctionHead>>) {
        if let Some(previous_callees) = self.callees.get(head) {
            for previous_callee in previous_callees.iter() {
                remove_from_multimap(&mut self.callers, previous_callee, head);
            }
        }
        for callee in new_callees.iter() {
            insert_into_multimap(&mut self.callers, Rc::clone(callee), Rc::clone(head));
        }
        self.callees.insert(Rc::clone(head), new_callees);
    }

    pub fn deep_calls<'a>(&self, from: impl Iterator<Item=&'a Rc<FunctionHead>>) -> LinkedHashSet<Rc<FunctionHead>> {
        let mut next = from.collect_vec();
        let mut gathered = LinkedHashSet::new();
        while let Some(current) = next.pop() {
            guard!(let Some(callees) = self.callees.get(current) else {
                continue  // Probably an internal function
            });
            gathered.extend(callees.iter().cloned());
            next.extend(callees.iter())
        }
        gathered
    }
}
