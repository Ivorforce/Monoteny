use crate::computation_tree;

pub trait Transpiler {
    fn transpile(&self, program: computation_tree::Program) -> String;
}
