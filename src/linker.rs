pub mod computation_tree;
pub mod scopes;
pub mod imperative;
pub mod global;

use crate::parser;
use crate::parser::abstract_syntax;
use crate::linker::computation_tree::*;
use crate::linker::global::link_file;
use crate::program::builtins::*;


pub fn link_program(syntax: abstract_syntax::Program, parser_scope: &parser::scopes::Level, scope: &scopes::Hierarchy, builtins: &TenLangBuiltins) -> Program {
    link_file(syntax, parser_scope, scope, builtins)
}
