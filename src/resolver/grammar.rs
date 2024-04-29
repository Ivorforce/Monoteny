use std::fmt::Display;
use std::hash::{Hash, Hasher};

use itertools::Itertools;

use crate::program::expression_tree::ExpressionID;
use crate::program::functions::ParameterKey;

pub mod precedence_order;
pub mod expressions;

#[derive(Clone)]
pub struct Struct {
    pub keys: Vec<ParameterKey>,
    pub values: Vec<ExpressionID>
}
