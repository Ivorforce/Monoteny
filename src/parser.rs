use crate::monoteny_grammar;

pub mod ast;

pub fn parse_program(content: &String) -> ast::Module {
    monoteny_grammar::ProgramParser::new()
        .parse(content.as_str())
        .unwrap()
}
