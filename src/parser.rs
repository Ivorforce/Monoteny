use lalrpop_util::ErrorRecovery;
use crate::error::{RResult, RuntimeError};
use crate::monoteny_grammar;

pub mod ast;
pub mod strings;
pub mod lexer;
pub mod error;

pub fn parse_program(content: &str) -> RResult<(ast::Module, Vec<ErrorRecovery<usize, lexer::Token<'_>, error::Error>>)> {
    let lexer = lexer::Lexer::new(content);
    let mut errors = vec![];
    let ast = monoteny_grammar::ProgramParser::new()
        .parse(&mut errors, content, lexer)
        .map_err(|e| RuntimeError::new(e.to_string()))?;

    Ok((ast, errors))
}
