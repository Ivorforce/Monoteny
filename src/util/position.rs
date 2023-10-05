use std::ops::Range;

#[derive(PartialEq, Eq, Clone)]
pub struct Positioned<V> {
    pub position: Range<usize>,
    pub value: V,
}

pub fn positioned<V>(v: V, start: usize, end: usize) -> Positioned<V> {
    Positioned {
        position: start..end,
        value: v,
    }
}
