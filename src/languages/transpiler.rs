use std::io::Write;
use crate::computation_tree;

pub trait Transpiler {
    fn transpile(&self, program: computation_tree::Program, stream: &mut (dyn Write)) -> Result<(), std::io::Error>;
}
