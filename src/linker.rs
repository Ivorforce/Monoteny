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
use crate::program::generics::TypeError;
use crate::program::Program;
use crate::program::traits::TraitConformanceError;


custom_error!{pub LinkError
    LinkError{msg: String} = "Linker Error: {msg}",
}


pub fn link_program(syntax: abstract_syntax::Program, parser_scope: &parser::scopes::Level, scope: &scopes::Hierarchy, builtins: &TenLangBuiltins) -> Result<Program, LinkError> {
    link_file(syntax, parser_scope, scope, builtins)
}

impl LinkError {
    pub fn map<T>(error: Result<T, TypeError>) -> Result<T, LinkError> {
        match error {
            Ok(t) => Ok(t),
            Err(err) => Err(LinkError::LinkError { msg: err.to_string() }),
        }
    }

    pub fn map_trait_error<T>(error: Result<T, TraitConformanceError>) -> Result<T, LinkError> {
        match error {
            Ok(t) => Ok(t),
            Err(err) => Err(LinkError::LinkError { msg: err.to_string() }),
        }
    }
}
