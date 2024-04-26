use annotate_snippets::Level;
use lalrpop_util::{ErrorRecovery, ParseError};

use crate::error::RuntimeError;
use crate::parser::lexer::Token;

#[derive(Debug, PartialEq, Clone)]
pub struct Error(pub String);

pub fn derive_error(error: &ErrorRecovery<usize, Token<'_>, Error>, start: usize, end: usize) -> RuntimeError {
    RuntimeError {
        level: Level::Error,
        path: None,
        range: Some(start..end),
        title: match &error.error {
            ParseError::InvalidToken { .. } => {
                format!("Invalid token")
            }
            ParseError::UnrecognizedEof { .. } => {
                format!("Unexpected end of file")
            }
            ParseError::UnrecognizedToken { token, expected } => {
                format!("Unexpected token: {}", token.1)
            }
            ParseError::ExtraToken { token } => {
                format!("Extraneous token: {}", token.1)
            }
            ParseError::User { error } => {
                format!("{}", error.0)
            }
        },
        notes: vec![],
    }
}
