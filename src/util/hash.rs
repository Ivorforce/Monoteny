use std::hash::{Hash, Hasher};

pub fn one(object: impl Hash, mut hasher: impl Hasher) -> u64 {
    object.hash(&mut hasher);
    hasher.finish()
}
