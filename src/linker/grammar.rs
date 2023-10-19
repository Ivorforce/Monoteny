use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use linked_hash_map::LinkedHashMap;
use strum::{Display, EnumIter};
use uuid::Uuid;
use crate::error::{RResult, RuntimeError};
use crate::parser::ast::PatternPart;
use crate::program::computation_tree::ExpressionID;
use crate::program::function_object::FunctionOverload;
use crate::program::functions::ParameterKey;

pub mod parse;
pub mod precedence_order;


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

#[derive(Clone)]
pub struct Struct {
    pub keys: Vec<ParameterKey>,
    pub values: Vec<ExpressionID>
}

#[derive(Clone)]
pub enum Token {
    Keyword(String),
    Expression(ExpressionID),
    AnonymousStruct(Struct),
    AnonymousArray { keys: Vec<Option<ExpressionID>>, values: Vec<ExpressionID> },
    FunctionReference { overload: Rc<FunctionOverload>, target: Option<ExpressionID> },
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Pattern {
    pub id: Uuid,
    pub alias: String,
    pub precedence_group: Rc<PrecedenceGroup>,

    pub parts: Vec<Box<PatternPart>>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Grammar {
    pub patterns: HashSet<Rc<Pattern>>,
    pub groups_and_keywords: LinkedHashMap<Rc<PrecedenceGroup>, HashMap<String, String>>,
}

impl Grammar {
    pub fn new() -> Grammar {
        Grammar {
            patterns: Default::default(),
            groups_and_keywords: Default::default(),
        }
    }

    pub fn set_precedence_order(&mut self, precedence: Vec<Rc<PrecedenceGroup>>) {
        self.groups_and_keywords = precedence.into_iter()
            .map(|p| (p, HashMap::new()))
            .collect();
        self.patterns = HashSet::new();
    }

    pub fn add_pattern(&mut self, pattern: Rc<Pattern>) -> RResult<Vec<String>> {
        guard!(let Some(keyword_map) = self.groups_and_keywords.get_mut(&pattern.precedence_group) else {
            panic!("Cannot find precedence group {:?} in: {:?}", pattern.precedence_group, self.groups_and_keywords);
        });

        let keywords = match &pattern.parts.iter().map(|x| x.as_ref()).collect_vec()[..] {
            [_] => return Err(RuntimeError::new(format!("Pattern is too short: {}.", pattern.alias))),
            [
                PatternPart::Keyword(keyword),
                PatternPart::Parameter { .. },
            ] => {
                assert_eq!(pattern.precedence_group.associativity, OperatorAssociativity::LeftUnary);
                keyword_map.insert(keyword.clone(), pattern.alias.clone());
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
                assert_ne!(pattern.precedence_group.associativity, OperatorAssociativity::LeftUnary);
                keyword_map.insert(keyword.clone(), pattern.alias.clone());
                vec![keyword.clone()]
            }
            _ => return Err(RuntimeError::new(String::from("This pattern form is not supported; try using unary or binary patterns."))),
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
