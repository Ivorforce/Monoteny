use std::fmt::{Display, Formatter};
use std::ops::Range;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Positioned<V> {
    pub position: Range<usize>,
    pub value: V,
}

impl<V> Positioned<V> {
    pub fn with_value<V1>(&self, v: V1) -> Positioned<V1> {
        Positioned {
            position: self.position.clone(),
            value: v,
        }
    }
}

pub fn positioned<V>(v: V, start: usize, end: usize) -> Positioned<V> {
    Positioned {
        position: start..end,
        value: v,
    }
}

impl<V: Display> Display for Positioned<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
