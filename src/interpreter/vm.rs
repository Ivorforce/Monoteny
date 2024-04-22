use crate::interpreter::chunks::{Chunk, Code};

pub struct VM {
    pub chunk: Chunk,
    pub stack: Vec<Code>,
}

impl VM {
    pub fn run(&mut self) {
        todo!()
    }
}
