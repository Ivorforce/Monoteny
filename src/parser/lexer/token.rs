use std::fmt::{Display, Formatter};

/// Token returned by the lexers
#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    Identifier(&'a str),
    MacroIdentifier(&'a str),
    OperatorIdentifier(&'a str),
    StringLiteral(String),
    IntLiteral(&'a str),
    RealLiteral(&'a str),
    Symbol(&'a str),
}

impl<'i> Display for Token<'i> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Identifier(s) => write!(f, "{}", s),
            Token::MacroIdentifier(s) => write!(f, "{}", s),
            Token::OperatorIdentifier(s) => write!(f, "{}", s),
            Token::IntLiteral(s) => write!(f, "{}", s),
            Token::RealLiteral(s) => write!(f, "{}", s),
            Token::Symbol(s) => write!(f, "{}", s),
            Token::StringLiteral(s) => write!(f, "{}", s),
        }
    }
}
