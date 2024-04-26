use itertools::Itertools;
use lalrpop_util::{ErrorRecovery, ParseError};

use crate::error::{RResult, RuntimeError};
use crate::monoteny_grammar;
use crate::parser::error::Error;
use crate::parser::lexer::Token;

pub mod ast;
pub mod strings;
pub mod lexer;
pub mod error;
mod tests;

fn rem_first_and_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}

pub fn parse_program(content: &str) -> RResult<(ast::Block, Vec<ErrorRecovery<usize, lexer::Token<'_>, error::Error>>)> {
    let lexer = lexer::Lexer::new(content);
    let mut errors = vec![];
    let ast = monoteny_grammar::BlockParser::new()
        .parse(&mut errors, content, lexer)
        .map_err(|e| {
            match e {
                ParseError::InvalidToken { location } => {
                    RuntimeError::error("Invalid token.").in_range(location..location)
                },
                ParseError::UnrecognizedEof { location, expected } => {
                    RuntimeError::error("File ended unexpectedly.").in_range(location..location)
                        .with_note(make_expected_note(expected))
                }
                ParseError::UnrecognizedToken { token: (start, token, end), expected } => {
                    RuntimeError::error("Unrecognized token.").in_range(start..end)
                        .with_note(make_expected_note(expected))
                }
                ParseError::ExtraToken { token: (start, token, end) } => {
                    RuntimeError::error("Extra token.").in_range(start..end)
                }
                ParseError::User { error } => {
                    panic!()
                }
            }.to_array()
        })?;

    Ok((ast, errors))
}

fn make_expected_note(expected: Vec<String>) -> RuntimeError {
    match &expected[..] {
        [one] => RuntimeError::note(format!("Expected: {}", rem_first_and_last(one)).as_str()),
        expected => RuntimeError::note(format!("Expected one of: {}", expected.iter().map(|s| rem_first_and_last(s)).join(" ")).as_str()),
    }
}
