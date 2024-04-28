use std::fmt::{Display, Error, Formatter};

use crate::ast::expression::Expression;

#[derive(Eq, PartialEq, Clone)]
pub struct Function {
    pub interface: FunctionInterface,
    pub body: Option<Expression>,
}

impl Display for Function {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "def {}", self.interface)?;

        if let Some(body) = &self.body {
            write!(fmt, " :: {}", body)?;
        }
        return Ok(())
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct FunctionInterface {
    pub expression: Expression,
    pub return_type: Option<Expression>,
}

impl Display for FunctionInterface {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", &self.expression)?;

        if let Some(return_type) = &self.return_type {
            write!(fmt, " -> {}", return_type)?;
        }

        Ok(())
    }
}
