use crate::interpreter::InterpreterError;
use crate::monoteny_grammar;

pub mod ast;
pub mod strings;
pub mod lexer;

pub fn parse_program(content: &str) -> Result<ast::Module, InterpreterError> {
    let lexer = lexer::Lexer::new(content);
    monoteny_grammar::ProgramParser::new()
        .parse(content, lexer)
        .map_err(|e| InterpreterError::ParserError { msg: e.to_string() })
}
