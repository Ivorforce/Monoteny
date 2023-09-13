use std::collections::{HashMap, HashSet};
use itertools::Itertools;
use uuid::Uuid;

pub struct Level {
    claims: HashMap<String, Vec<Uuid>>,
    keywords: HashSet<Uuid>,
    sublevels: Vec<Level>
}

impl Level {
    pub fn new() -> Level {
        Level {
            claims: HashMap::new(),
            keywords: HashSet::new(),
            sublevels: Vec::new(),
        }
    }

    pub fn register_definition(&mut self, uuid: Uuid, name: &str) {
        if let Some(existing) = self.claims.get_mut(name) {
            existing.push(uuid);
        }
        else {
            self.claims.insert(
                name.to_string(),
                vec![uuid.clone()]
            );
        }
    }

    pub fn insert_keyword(&mut self, uuid: Uuid, name: &str) {
        self.register_definition(uuid, name);
        self.keywords.insert(uuid);
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
            if let [claim] = claims[..] {
                // Can use plain name
                let mut name = name.clone();
                while reserved.contains(&name) {
                    name = format!("{}_", name);
                }
                reserved.insert(name.clone());
                mapping.insert(claim, name);
            }
            else {
                // Need to postfix each name with an idx
                let mut prefix = format!("{}_", name);
                while (0 .. claims.len()).any(|idx| reserved.contains(&make_name(&prefix, idx))) {
                    prefix = format!("{}_", prefix);
                }

                let mut idx = 0;
                for claim in claims.iter() {
                    if self.keywords.contains(claim) {
                        reserved.insert(name.clone());
                        mapping.insert(claim.clone(), name.clone());
                    }
                    else {
                        let postfixed_name = make_name(&prefix, idx);
                        reserved.insert(postfixed_name.clone());

                        mapping.insert(claim.clone(), postfixed_name);
                        idx += 1;
                    }
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
