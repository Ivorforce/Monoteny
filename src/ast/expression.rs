use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};

use crate::ast::term::Term;
use crate::error::{RResult, TryCollectMany};
use crate::util::fmt::write_separated_display;
use crate::util::position::Positioned;

#[derive(Eq, PartialEq, Clone)]
pub struct Expression(Vec<Box<Positioned<Term>>>);

impl Expression {
    pub fn no_errors(&self) -> RResult<()> {
        self.iter()
            .map(|t| match &t.value {
                Term::Error(e) => Err(e.clone().to_array()),
                _ => Ok(())
            })
            .try_collect_many()
    }
}

impl Deref for Expression {
    type Target = Vec<Box<Positioned<Term>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Expression {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<Box<Positioned<Term>>>> for Expression {
    fn from(value: Vec<Box<Positioned<Term>>>) -> Self {
        Expression(value)
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_separated_display(f, " ", self.0.iter().map(|b| &b.value))
    }
}
