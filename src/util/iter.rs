use std::collections::VecDeque;
use itertools::Itertools;

pub fn omega<E, I: Iterator<Item=E>, I1: Iterator<Item=E>>(e: I, mut deeper: impl FnMut(&E) -> I1) -> Vec<E> {
     let mut all = e.collect_vec();
     let mut next: VecDeque<_> = (0..all.len()).into_iter().collect();
     while let Some(current) = next.pop_front() {
          let size_before = all.len();
          all.extend(deeper(&all[current]));
          let added_count = all.len() - size_before;
          next.extend(&all.len() - added_count..all.len());
     }
     all
}
