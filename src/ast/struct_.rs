use std::fmt::{Display, Error, Formatter};

use crate::ast::Expression;
use crate::program::functions::ParameterKey;
use crate::util::fmt::write_separated_display;
use crate::util::position::Positioned;

#[derive(Eq, PartialEq, Clone)]
pub struct Struct { pub arguments: Vec<Box<Positioned<StructArgument>>> }

impl Struct {
    pub fn empty() -> Struct {
        Struct { arguments: vec![] }
    }
}

impl Display for Struct {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        write_separated_display(f, ", ", self.arguments.iter().map(|f| &f.value))?;
        write!(f, ")")
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct StructArgument {
    pub key: ParameterKey,
    pub value: Expression,
    pub type_declaration: Option<Expression>,
}

impl Display for StructArgument {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{}{}", self.key, self.value)
    }
}
