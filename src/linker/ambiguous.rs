use std::fmt::Display;
use itertools::Itertools;
use crate::linker::imperative::ImperativeLinker;
use crate::linker::LinkError;

pub mod function_call;
pub mod number_literal;

pub use function_call::{AmbiguousFunctionCall, AmbiguousFunctionCandidate};
pub use number_literal::AmbiguousNumberLiteral;

pub trait LinkerAmbiguity: Display {
    fn attempt_to_resolve(&mut self, expressions: &mut ImperativeLinker) -> Result<bool, LinkError>;
}
