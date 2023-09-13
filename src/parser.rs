use crate::monoteny_grammar;

pub mod ast;
pub mod strings;
pub mod lexer;

pub fn parse_program(content: &String) -> ast::Module {
    let lexer = lexer::Lexer::new(content);
    monoteny_grammar::ProgramParser::new()
        .parse(content.as_str(), lexer)
        .unwrap()
}
