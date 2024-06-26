use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Error, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use itertools::Itertools;
use linked_hash_map::LinkedHashMap;
use strum::{Display, EnumIter};
use uuid::Uuid;

use crate::error::{RResult, RuntimeError};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Display, EnumIter)]
pub enum OperatorAssociativity {
    LeftUnary,  // Evaluated with the operator left of the expression.
    RightUnary,  // Evaluated with the operator right of the expression.
    Left,  // Left evaluated first.
    Right, // Right evaluated first.
    None,  // Fail parsing if more than one neighboring operator is found.
    LeftConjunctivePairs, // Evaluated in pairs left first, joined by && operations.
}

#[derive(Eq, Debug)]
pub struct PrecedenceGroup {
    pub trait_id: Uuid,
    pub name: String,
    pub associativity: OperatorAssociativity,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Pattern<Function: Clone + PartialEq + Eq + Hash + Debug> {
    pub id: Uuid,
    pub precedence_group: Rc<PrecedenceGroup>,

    pub parts: Vec<Box<PatternPart>>,
    pub function: Function,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum PatternPart {
    Parameter(usize),
    Keyword(String),
}

#[derive(Clone, PartialEq, Eq)]
pub struct Grammar<Function: Clone + PartialEq + Eq + Hash + Debug> {
    pub patterns: HashSet<Rc<Pattern<Function>>>,
    pub keywords: HashSet<String>,
    pub groups_and_keywords: LinkedHashMap<Rc<PrecedenceGroup>, HashMap<String, Function>>,
}

impl<Function: Clone + PartialEq + Eq + Hash + Debug> Grammar<Function> where  {
    pub fn new() -> Self {
        Grammar {
            patterns: Default::default(),
            keywords: Default::default(),
            groups_and_keywords: Default::default(),
        }
    }

    pub fn set_precedence_order(&mut self, precedence: Vec<Rc<PrecedenceGroup>>) {
        self.groups_and_keywords = precedence.into_iter()
            .map(|p| (p, HashMap::new()))
            .collect();
        self.patterns = HashSet::new();
        self.keywords = HashSet::new();
    }

    pub fn add_pattern(&mut self, pattern: Rc<Pattern<Function>>) -> RResult<Vec<String>> {
        let Some(keyword_map) = self.groups_and_keywords.get_mut(&pattern.precedence_group) else {
            panic!("Cannot find precedence group {:?} in: {:?}", pattern.precedence_group, self.groups_and_keywords);
        };

        let keywords = match &pattern.parts.iter().map(|x| x.as_ref()).collect_vec()[..] {
            [_] => return Err(RuntimeError::error("Pattern is too short.").to_array()),
            [
                PatternPart::Keyword(keyword),
                PatternPart::Parameter { .. },
            ] => {
                if pattern.precedence_group.associativity != OperatorAssociativity::LeftUnary {
                    return Err(RuntimeError::error("Unary pattern must use LeftUnary precedence.").to_array())
                }
                keyword_map.insert(keyword.clone(), pattern.function.clone());
                self.keywords.insert(keyword.clone());
                vec![keyword.clone()]
            },
            [
                PatternPart::Parameter { .. },
                PatternPart::Keyword(keyword),
            ] => {
                todo!("Right unary operators aren't supported yet.")
            },
            [
                PatternPart::Parameter { .. },
                PatternPart::Keyword(keyword),
                PatternPart::Parameter { .. },
            ] => {
                if pattern.precedence_group.associativity == OperatorAssociativity::LeftUnary {
                    return Err(RuntimeError::error("Binary pattern must not use LeftUnary precedence.").to_array())
                }

                keyword_map.insert(keyword.clone(), pattern.function.clone());
                self.keywords.insert(keyword.clone());
                vec![keyword.clone()]
            }
            _ => return Err(RuntimeError::error("This pattern form is not supported; try using unary or binary patterns.").to_array()),
        };

        self.patterns.insert(pattern);
        Ok(keywords)
    }
}

impl PrecedenceGroup {
    pub fn new(name: &str, associativity: OperatorAssociativity) -> PrecedenceGroup {
        PrecedenceGroup {
            trait_id: Uuid::new_v4(),
            name: String::from(name),
            associativity,
        }
    }
}

impl PartialEq for PrecedenceGroup {
    fn eq(&self, other: &Self) -> bool {
        self.trait_id == other.trait_id
    }
}

impl Hash for PrecedenceGroup {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.trait_id.hash(state)
    }
}


impl Display for PatternPart {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            PatternPart::Parameter(p) => write!(fmt, "({})", p),
            PatternPart::Keyword(keyword) => write!(fmt, "{}", keyword),
        }
    }
}
