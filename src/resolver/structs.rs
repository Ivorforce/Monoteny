use std::fmt::Display;
use std::hash::Hash;

use crate::program::expression_tree::ExpressionID;
use crate::program::functions::ParameterKey;

#[derive(Clone)]
pub struct AnonymousStruct {
    pub keys: Vec<ParameterKey>,
    pub values: Vec<ExpressionID>
}
