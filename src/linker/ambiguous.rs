use std::fmt::Display;
use crate::linker::imperative::ImperativeLinker;

pub mod function_call;
pub mod abstract_call;

pub use function_call::{AmbiguousFunctionCall, AmbiguousFunctionCandidate};
pub use abstract_call::AmbiguousAbstractCall;

use crate::interpreter::InterpreterError;
pub enum AmbiguityResult<V> {
    Ok(V),
    Ambiguous,
}

pub trait LinkerAmbiguity: Display {
    fn attempt_to_resolve(&mut self, expressions: &mut ImperativeLinker) -> Result<AmbiguityResult<()>, InterpreterError>;
}
