use std::fmt::{Display, Formatter};

use crate::ast::{Array, Expression};
use crate::error::{RResult, RuntimeError, TryCollectMany};

#[derive(PartialEq, Eq, Clone)]
pub struct Decorated<T> {
    pub decorations: Array,
    pub value: T,
}

impl<V> Decorated<V> {
    pub fn with_value<N>(&self, n: N) -> Decorated<N> {
        Decorated {
            decorations: self.decorations.clone(),
            value: n,
        }
    }

    pub fn undecorated(t: V) -> Decorated<V> {
        Decorated {
            decorations: Array { arguments: vec![] },
            value: t,
        }
    }

    pub fn decorations_as_vec(&self) -> RResult<Vec<&Expression>> {
        return self.decorations.arguments.iter().map(|d| {
            if d.value.key.is_some() {
                return Err(RuntimeError::error("Decorations cannot have keys.").to_array())
            }
            if d.value.type_declaration.is_some() {
                return Err(RuntimeError::error("Decorations cannot have type declarations.").to_array())
            }

            Ok(&d.value.value)
        }).try_collect_many()
    }

    pub fn no_decorations(&self) -> RResult<()> {
        if !self.decorations.arguments.is_empty() {
            return Err(RuntimeError::error("Decorations are not supported in this context.").to_array())
        }

        return Ok(())
    }
}

impl<V: Display> Display for Decorated<V> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        if self.decorations.arguments.is_empty() {
            return write!(fmt, "{}", self.value)
        }
        write!(fmt, "!{}\n{}", self.decorations, self.value)
    }
}
