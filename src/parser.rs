use itertools::Itertools;
use lalrpop_util::ErrorRecovery;

use crate::error::RResult;
use crate::monoteny_grammar;

pub mod ast;
pub mod strings;
pub mod lexer;
pub mod error;
mod tests;

pub fn parse_program(content: &str) -> RResult<(ast::Block, Vec<ErrorRecovery<usize, lexer::Token<'_>, error::Error>>)> {
    let lexer = lexer::Lexer::new(content);
    let mut errors = vec![];
    let ast = monoteny_grammar::FileParser::new()
        .parse(&mut errors, content, lexer)
        .map_err(|e| { error::map_parse_error(&e).to_array() })?;

    Ok((ast, errors))
}
