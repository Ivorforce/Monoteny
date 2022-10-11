use std::rc::Rc;
use uuid::Uuid;
use crate::monoteny_grammar;

pub mod abstract_syntax;

use abstract_syntax::*;
use crate::program::types::Pattern;
use crate::program::builtins::Builtins;

pub fn parse_program(content: &String) -> Program {
    monoteny_grammar::ProgramParser::new()
        .parse(content.as_str())
        .unwrap()
}
