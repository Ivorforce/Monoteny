use std::fmt::{Display, Formatter};

use crate::ast::Block;

#[derive(Eq, PartialEq, Clone)]
pub struct TraitDefinition {
    pub name: String,
    pub block: Box<Block>,
}

impl Display for TraitDefinition {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "trait {} {{\n{}}}", self.name, self.block)
    }
}
