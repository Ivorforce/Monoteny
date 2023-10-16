use std::collections::{HashMap, HashSet};
use itertools::Itertools;
use linked_hash_set::LinkedHashSet;
use uuid::Uuid;

#[derive(Clone)]
pub struct Level {
    claims: HashMap<String, LinkedHashSet<Uuid>>,
    sublevels: Vec<Level>
}

impl Level {
    pub fn new() -> Level {
        Level {
            claims: HashMap::new(),
            sublevels: Vec::new(),
        }
    }

    pub fn insert_name(&mut self, uuid: Uuid, name: &str) -> Uuid {
        if let Some(existing) = self.claims.get_mut(name) {
            existing.insert_if_absent(uuid);
        }
        else {
            self.claims.insert(
                name.to_string(),
                LinkedHashSet::from_iter([uuid.clone()])
            );
        }

        uuid
    }

    pub fn add_sublevel(&mut self) -> &mut Level {
        let new_level = Level::new();
        self.sublevels.push(new_level);
        self.sublevels.last_mut().unwrap()
    }

    fn insert_names(&self, mapping: &mut HashMap<Uuid, String>, reserved: &HashSet<String>) {
        let mut reserved = reserved.clone();

        fn make_name(prefix: &str, idx: usize) -> String {
            format!("{}{}", prefix, idx)
        }

        for (name, claims) in self.claims.iter().sorted_by_key(|(name, claims)| name.len()) {
            if let Ok(claim) = claims.iter().exactly_one() {
                // Can use plain name
                let mut name = name.clone();
                while reserved.contains(&name) {
                    name = format!("{}_", name);
                }
                reserved.insert(name.clone());
                mapping.insert(*claim, name);
            }
            else {
                // Need to postfix each name with an idx
                let mut prefix = format!("{}_", name);
                while (0 .. claims.len()).any(|idx| reserved.contains(&make_name(&prefix, idx))) {
                    prefix = format!("{}_", prefix);
                }

                let mut idx = 0;
                for claim in claims.iter() {
                    let postfixed_name = make_name(&prefix, idx);
                    reserved.insert(postfixed_name.clone());

                    mapping.insert(claim.clone(), postfixed_name);
                    idx += 1;
                }
            }
        }

        for level in self.sublevels.iter() {
            level.insert_names(mapping, &reserved);
        }
    }

    pub fn map_names(&self) -> HashMap<Uuid, String> {
        let mut map = HashMap::new();
        self.insert_names(&mut map, &HashSet::new());
        map
    }
}
