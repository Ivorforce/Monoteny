use std::fmt::{Display, Formatter};

use crate::ast::Block;

#[derive(Eq, PartialEq, Clone)]
pub struct StructDefinition {
    pub name: String,
    pub block: Box<Block>,
}

impl Display for StructDefinition {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "struct {} {{\n{}}}", self.name, self.block)
    }
}
