use std::fmt::Display;
use crate::linker::imperative::ImperativeLinker;
use crate::linker::LinkError;

pub mod function_call;
pub mod abstract_call;

pub use function_call::{AmbiguousFunctionCall, AmbiguousFunctionCandidate};
pub use abstract_call::AmbiguousAbstractCall;

pub trait LinkerAmbiguity: Display {
    fn attempt_to_resolve(&mut self, expressions: &mut ImperativeLinker) -> Result<bool, LinkError>;
}
