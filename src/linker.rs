pub mod scopes;
pub mod imperative;
pub mod global;
pub mod ambiguous;
pub mod precedence;
pub mod r#type;
pub mod interface;

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


pub fn link_program(syntax: abstract_syntax::Program, scope: &scopes::Scope, builtins: &Builtins) -> Result<Program, LinkError> {
    link_file(syntax, scope, builtins)
}
