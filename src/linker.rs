pub mod scopes;
pub mod imperative;
pub mod global;
pub mod ambiguous;

use custom_error::custom_error;
use crate::parser;
use crate::parser::abstract_syntax;
use crate::program::computation_tree::*;
use crate::linker::global::link_file;
use crate::program::builtins::*;
use crate::program::Program;


custom_error!{pub LinkError
    LinkError{msg: String} = "Linker Error: {msg}",
    Ambiguous = "Ambiguous",
}


pub fn link_program(syntax: abstract_syntax::Program, parser_scope: &parser::scopes::Level, scope: &scopes::Hierarchy, builtins: &Builtins) -> Result<Program, LinkError> {
    link_file(syntax, parser_scope, scope, builtins)
}
