use crate::monoteny_grammar;

pub mod abstract_syntax;

use abstract_syntax::*;

pub fn parse_program(content: &String) -> Program {
    monoteny_grammar::ProgramParser::new()
        .parse(content.as_str())
        .unwrap()
}
