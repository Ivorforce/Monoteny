use std::fmt::{Display, Error, Formatter};

use crate::ast::{Array, Block, Expression, StringPart, Struct};
use crate::error::RuntimeError;
use crate::util::position::Positioned;

#[derive(Eq, PartialEq, Clone)]
pub enum Term {
    Error(RuntimeError),
    Identifier(String),
    MacroIdentifier(String),
    Dot,
    IntLiteral(String),
    RealLiteral(String),
    Struct(Box<Struct>),
    Array(Box<Array>),
    StringLiteral(Vec<Box<Positioned<StringPart>>>),
    Block(Box<Block>),
    IfThenElse(Box<IfThenElse>),
}

impl Display for Term {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Term::Error(err) => write!(fmt, "ERR"),
            Term::Identifier(s) => write!(fmt, "{}", s),
            Term::MacroIdentifier(s) => write!(fmt, "{}!", s),
            Term::IntLiteral(s) => write!(fmt, "{}", s),
            Term::RealLiteral(s) => write!(fmt, "{}", s),
            Term::StringLiteral(parts) => {
                write!(fmt, "\"")?;
                for part in parts {
                    write!(fmt, "{}", part)?;
                }
                write!(fmt, "\"")
            },
            Term::Struct(struct_) => write!(fmt, "{}", struct_),
            Term::Array(array) => write!(fmt, "{}", array),
            Term::Block(block) => write!(fmt, "{{\n{}}}", block),
            Term::Dot => write!(fmt, "."),
            Term::IfThenElse(if_then_else) => {
                write!(fmt, "if {} :: {}", if_then_else.condition, if_then_else.consequent)?;
                if let Some(alternative) = &if_then_else.alternative {
                    write!(fmt, "else :: {}", alternative)?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct IfThenElse {
    pub condition: Expression,
    pub consequent: Expression,
    pub alternative: Option<Expression>,
}

