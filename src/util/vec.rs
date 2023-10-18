use itertools::Itertools;

pub fn swizzle<A>(vec: &mut Vec<A>, swizzle: &Vec<usize>) -> Vec<A> {
    if swizzle.is_empty() {
        return vec.drain(..).collect_vec()
    }

    assert!(swizzle.iter().duplicates().next().is_none());

    // TODO Would be nicer with unsafe, but eh.
    let mut tmp_array = vec.drain(..).enumerate()
        .map(|(idx, obj)| (swizzle.iter().position(|p| p == &idx).unwrap_or(usize::MAX), obj))
        .collect_vec();
    tmp_array.sort_by_key(|(idx, obj)| *idx);
    let mut removed = vec![];
    for (idx, obj) in tmp_array {
        if idx != usize::MAX {
            vec.push(obj)
        }
        else {
            removed.push(obj)
        }
    }
    removed
}
