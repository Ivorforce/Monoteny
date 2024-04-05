use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use itertools::Itertools;
use crate::util::iter::omega;
use crate::util::vec;


/// TODO We could also use an actual tree...
#[derive(Clone)]
pub struct NodeTree<Key: Hash + Eq + Clone, Value> {
    pub root: Key,
    /// Will be set for every expression ID
    pub children: HashMap<Key, Vec<Key>>,
    /// Will be set for every expression ID
    pub parents: HashMap<Key, Key>,
    /// Might not be set for a while
    pub values: HashMap<Key, Value>,
}

impl<Key: Hash + Eq + Clone, Value> NodeTree<Key, Value> {
    pub fn new(root: Key) -> NodeTree<Key, Value> {
        NodeTree {
            root,
            values: HashMap::new(),
            children: HashMap::new(),
            parents: Default::default(),
        }
    }

    pub fn deep_children(&self, start: Key) -> Vec<Key> {
        omega([start].into_iter(), |e| self.children[e].iter().cloned()).collect_vec()
    }

    pub fn truncate_up_and_down(&mut self, mut trim: Vec<Key>, up_until: impl Fn(&Value) -> bool) {
        while let Some(current) = trim.pop() {
            let Entry::Occupied(o) = self.values.entry(current) else {
                continue // TODO This shouldn't happen but it does rn for some reason. Maybe because of the other truncations?
            };

            if up_until(o.get()) {
                self.children.get_mut(o.key()).unwrap().retain(|a| self.values.contains_key(a))
            }
            else {
                trim.push(self.parents.remove(o.key()).unwrap());
                trim.extend(self.children.remove(o.key()).unwrap());
                o.remove();
            }
        }
    }

    pub fn truncate_down(&mut self, mut include: Vec<Key>) {
        while let Some(current) = include.pop() {
            // It's enough to remove arguments and operations.
            // The type forest may still contain orphans, but that's ok. And our type doesn't change
            //  from inlining.
            self.values.remove(&current);
            self.parents.remove(&current);
            include.extend(self.children.remove(&current).unwrap());
        }
    }

    pub fn swizzle_arguments(&mut self, expression_id: Key, swizzle: &Vec<usize>) {
        let args = self.children.get_mut(&expression_id).unwrap();
        let removed = vec::swizzle(args, swizzle);
        self.truncate_down(removed);
    }

    pub fn inline(&mut self, expression_id: Key, parameter_idx: usize) {
        let mut arguments_before = self.children[&expression_id].clone();

        let replacement_id = arguments_before.remove(parameter_idx);
        let replacement_operation = self.values.remove(&replacement_id).unwrap();
        let replacement_arguments = self.children.remove(&replacement_id).unwrap();

        for arg in replacement_arguments.iter() {
            *self.parents.get_mut(arg).unwrap() = expression_id.clone();
        }

        let operation = self.values.get_mut(&expression_id).unwrap();
        *operation = replacement_operation;

        let arguments = self.children.get_mut(&expression_id).unwrap();
        *arguments = replacement_arguments;

        self.truncate_down(arguments_before);
    }
}
