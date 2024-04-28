use std::fmt::{Display, Error, Formatter};

use crate::ast::Expression;
use crate::util::fmt::write_separated_display;
use crate::util::position::Positioned;

#[derive(Eq, PartialEq, Clone)]
pub struct Array { pub arguments: Vec<Box<Positioned<ArrayArgument>>> }

impl Array {
    pub fn empty() -> Array {
        Array { arguments: vec![] }
    }
}

impl Display for Array {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        write_separated_display(f, ", ", self.arguments.iter().map(|f| &f.value))?;
        write!(f, "]")
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct ArrayArgument {
    pub key: Option<Expression>,
    pub value: Expression,
    pub type_declaration: Option<Expression>,
}

impl Display for ArrayArgument {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        if let Some(key) = &self.key {
            write!(fmt, "{}: ", key)?;
        }
        write!(fmt, "{}", self.value)
    }
}
