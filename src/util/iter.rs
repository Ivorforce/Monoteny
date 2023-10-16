use std::collections::VecDeque;

pub struct Omega<E, I: Iterator<Item=E>, F: FnMut(&E) -> I> {
     pub deeper: F,
     pub next: VecDeque<E>,
}

impl<E, I: Iterator<Item=E>, F: FnMut(&E) -> I> Iterator for Omega<E, I, F> {
     type Item = E;

     fn next(&mut self) -> Option<Self::Item> {
          self.next.pop_front().map(|current| {
               let next = (self.deeper)(&current);
               self.next.extend(next);
               current
          })
     }
}

pub fn omega<E, I0: Iterator<Item=E>, I: Iterator<Item=E>, F: FnMut(&E) -> I>(start: I0, mut deeper: F) -> Omega<E, I, F> {
     Omega {
          deeper,
          next: VecDeque::from_iter(start),
     }
}
