use std::fmt::{Display, Formatter};
use itertools::Itertools;
use lalrpop_util::{ErrorRecovery, ParseError};

use crate::error::RuntimeError;
use crate::parser::lexer::Token;
use crate::util::position::Positioned;

#[derive(Debug, PartialEq, Clone)]
pub struct Error(pub String);

pub fn derive_error(error: &Positioned<ErrorRecovery<usize, Token<'_>, Error>>) -> RuntimeError {
    map_parse_error(&error.value.error)
        .in_range(error.position.clone())
}

pub fn map_parse_error(e: &ParseError<usize, Token, Error>) -> RuntimeError {
    match e {
        ParseError::InvalidToken { location } => {
            RuntimeError::error("Invalid token.").in_range(*location..*location)
        },
        ParseError::UnrecognizedEof { location, expected } => {
            add_expected_note(RuntimeError::error("File ended unexpectedly.").in_range(*location..*location), expected)
        }
        ParseError::UnrecognizedToken { token: (start, token, end), expected } => {
            add_expected_note(RuntimeError::error("Unexpected token.").in_range(*start..*end), expected)
        }
        ParseError::ExtraToken { token: (start, token, end) } => {
            RuntimeError::error("Extraneous token.").in_range(*start..*end)
        }
        ParseError::User { error } => {
            panic!()
        }
    }
}

fn unquote(value: &str) -> &str {
    if !value.starts_with('\"') {
        return value
    }

    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}

fn add_expected_note(error: RuntimeError, expected: &Vec<String>) -> RuntimeError {
    match &expected[..] {
        [] => error,
        [one] => error.with_note(RuntimeError::note(format!("Expected: {}", unquote(one)).as_str())),
        expected => error.with_note(RuntimeError::note(format!("Expected one of: {}", expected.iter().map(|s| unquote(s)).join(" ")).as_str())),
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
