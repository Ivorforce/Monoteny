use std::fmt::Display;

pub use abstract_call::AmbiguousAbstractCall;
pub use function_call::{AmbiguousFunctionCall, AmbiguousFunctionCandidate};

use crate::error::RResult;
use crate::linker::imperative::ImperativeLinker;

pub mod function_call;
pub mod abstract_call;

pub enum AmbiguityResult<V> {
    Ok(V),
    Ambiguous,
}

pub trait LinkerAmbiguity: Display {
    fn attempt_to_resolve(&mut self, expressions: &mut ImperativeLinker) -> RResult<AmbiguityResult<()>>;
}
