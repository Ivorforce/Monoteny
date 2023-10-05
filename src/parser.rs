use lalrpop_util::ErrorRecovery;
use crate::error::RuntimeError;
use crate::monoteny_grammar;

pub mod ast;
pub mod strings;
pub mod lexer;

pub fn parse_program(content: &str) -> Result<(ast::Module, Vec<ErrorRecovery<usize, lexer::Token<'_>, lexer::Error>>), RuntimeError> {
    let lexer = lexer::Lexer::new(content);
    let mut errors = vec![];
    let ast = monoteny_grammar::ProgramParser::new()
        .parse(&mut errors, content, lexer)
        .map_err(|e| RuntimeError { msg: e.to_string() })?;

    Ok((ast, errors))
}
