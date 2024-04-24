use lalrpop_util::{ErrorRecovery, ParseError};

use crate::error::{FilePosition, RuntimeError};
use crate::parser::lexer::Token;

#[derive(Debug, PartialEq, Clone)]
pub struct Error(pub String);

pub fn derive_error(error: &ErrorRecovery<usize, Token<'_>, Error>, start: usize, end: usize) -> RuntimeError {
    RuntimeError {
        position: FilePosition {
            file: None,
            range: Some(start..end),
        },
        msg: match &error.error {
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
    }
}
