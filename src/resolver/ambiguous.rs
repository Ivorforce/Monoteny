use std::fmt::Display;
use std::ops::Range;

pub use abstract_call::AmbiguousAbstractCall;
pub use function_call::{AmbiguousFunctionCall, AmbiguousFunctionCandidate};

use crate::error::RResult;
use crate::resolver::imperative::ImperativeResolver;

pub mod function_call;
pub mod abstract_call;

pub enum AmbiguityResult<V> {
    Ok(V),
    Ambiguous,
}

pub trait ResolverAmbiguity: Display {
    fn attempt_to_resolve(&mut self, expressions: &mut ImperativeResolver) -> RResult<AmbiguityResult<()>>;

    fn get_position(&self) -> Range<usize>;
}
