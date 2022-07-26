use crate::computation_tree;
use crate::computation_tree::Program;

use crate::languages::transpiler::Transpiler;

pub struct PythonTranspiler {

}

impl Transpiler for PythonTranspiler {
    fn transpile(&self, program: Program) -> String {
        return String::from("")
    }
}
