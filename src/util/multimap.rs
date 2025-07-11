use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub fn insert_into_multimap<K, V>(multimap: &mut HashMap<K, HashSet<V>>, key: K, value: V) where K: Hash, V: Hash, K: Eq, V: Eq, V: Clone, K: Clone {
    match multimap.entry(key) {
        Entry::Occupied(o) => {
            o.into_mut().insert(value);
        }
        Entry::Vacant(v) => {
            v.insert(HashSet::from([value]));
        }
    }
}

pub fn remove_from_multimap<K, V>(multimap: &mut HashMap<K, HashSet<V>>, key: &K, value: &V) where K: Hash, V: Hash, K: Eq, V: Eq, V: Clone, K: Clone {
    if let Some(map) = multimap.get_mut(key) {
        map.remove(value);
    }
}
