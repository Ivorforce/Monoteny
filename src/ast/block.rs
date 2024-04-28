use std::fmt::{Display, Error, Formatter};

use crate::ast::decorated::Decorated;
use crate::ast::Statement;
use crate::util::fmt::write_separated_display;
use crate::util::position::Positioned;

#[derive(Eq, PartialEq, Clone)]
pub struct Block {
    pub statements: Vec<Box<Decorated<Positioned<Statement>>>>
}

impl Display for Block {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write_separated_display(fmt, "\n", self.statements.iter())
    }
}
