use uuid::Uuid;
use std::hash::{Hash, Hasher};

pub struct Struct {
    pub id: Uuid,
    pub name: String,
}

impl PartialEq for Struct {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Struct {}

impl Hash for Struct {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
