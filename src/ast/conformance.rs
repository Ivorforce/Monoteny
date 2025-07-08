use std::fmt::{Display, Formatter};

use crate::ast::expression::Expression;
use crate::ast::Block;

#[derive(Eq, PartialEq, Clone)]
pub struct TraitConformanceDeclaration {
    pub declared_for: Expression,
    pub declared: Expression,
    pub block: Box<Block>,
}

impl Display for TraitConformanceDeclaration {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "declare {} is {} {{}} :: {{\n{}}}", self.declared_for, self.declared, self.block)
    }
}
