use std::collections::HashMap;
use std::hash::Hash;

pub fn extend_multimap<K, V>(multimap: &mut HashMap<K, Vec<V>>, extension: &HashMap<K, Vec<V>>) where K: Hash, K: Eq, V: Clone, K: Clone {
    for (trait_, trait_declarations) in extension.iter() {
        if let Some(existing) = multimap.get_mut(trait_) {
            existing.extend(trait_declarations.clone());
        }
        else {
            multimap.insert(trait_.clone(), trait_declarations.clone());
        }
    }
}

pub fn push_into_multimap<K, V>(multimap: &mut HashMap<K, Vec<V>>, key: &K, value: V) where K: Hash, K: Eq, V: Clone, K: Clone {
    if let Some(existing) = multimap.get_mut(key) {
        existing.push(value);
    }
    else {
        multimap.insert(key.clone(), vec![value]);
    }
}
