pub fn omega<E, I: Iterator<Item=E>, I1: Iterator<Item=E>>(e: I, deeper: &dyn Fn(&E) -> I1) -> Vec<E> {
     e.map(|e| {
          let c = deeper(&e);
          [e].into_iter().chain(omega(c, deeper))
     }).flatten().collect()
}
