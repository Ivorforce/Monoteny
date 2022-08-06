use std::collections::HashSet;
use std::rc::Rc;
use crate::parser::associativity::BinaryPrecedenceGroup;

pub struct Level {
    pub precedence_groups: Vec<(Rc<BinaryPrecedenceGroup>, HashSet<String>)>,
}

impl Level {
    pub fn new() -> Level {
        Level {
            precedence_groups: vec![]
        }
    }

    pub fn sublevel(&self) -> Level {
        Level {
            precedence_groups: self.precedence_groups.clone()
        }
    }

    pub fn add_binary_pattern(&mut self, operator: String, precedence_group: &Rc<BinaryPrecedenceGroup>) {
        let mut found_group = false;

        for (other_group, set) in self.precedence_groups.iter_mut() {
            if other_group != precedence_group {
                if set.contains(&operator) {
                    panic!("Duplicate, incompatible pattern definition for {}", &operator);
                }

                continue
            }

            found_group = true;
            set.insert(operator.clone());
        }

        if !found_group {
            panic!("Unknown precedence group: {}", &precedence_group.name);
        }
    }
}
