use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::parser::associativity::{OperatorAssociativity, PrecedenceGroup};
use crate::program::types::Pattern;

pub struct Level {
    pub precedence_groups: Vec<(Rc<PrecedenceGroup>, HashSet<String>)>,
    pub patterns: HashSet<Pattern>,
}

impl Level {
    pub fn new() -> Level {
        Level {
            precedence_groups: vec![],
            patterns: HashSet::new(),
        }
    }

    pub fn sublevel(&self) -> Level {
        Level {
            precedence_groups: self.precedence_groups.clone(),
            patterns: HashSet::new(),
        }
    }

    pub fn resolve_operator_pattern(&self, operator_name: &String, is_binary: bool) -> &Pattern {
        for pattern in self.patterns.iter() {
            if pattern.precedence_group.is_binary() == is_binary && operator_name == &pattern.operator {
                return pattern
            }
        }

        panic!("Patter could not be resolved for operator: {}", operator_name)
    }

    pub fn resolve_precedence_group(&self, name: &String) -> Rc<PrecedenceGroup> {
        for (group, _) in self.precedence_groups.iter() {
            if &group.name == name {
                return Rc::clone(group)
            }
        }

        panic!("Precedence group could not be resolved: {}", name)
    }

    pub fn add_pattern(&mut self, pattern: Pattern) {
        for (other_group, set) in self.precedence_groups.iter_mut() {
            if other_group == &pattern.precedence_group {
                set.insert(pattern.operator.clone());
                continue
            }
        }

        self.patterns.insert(pattern);
    }
}
