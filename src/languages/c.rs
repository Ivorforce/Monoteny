use std::io::Write;
use crate::computation_tree;
use crate::computation_tree::Program;

use crate::languages::transpiler::Transpiler;

pub struct CTranspiler {

}

impl Transpiler for CTranspiler {
    fn transpile(&self, program: &Program, stream: &mut (dyn Write)) -> Result<(), std::io::Error> {
        for function in program.functions.iter() {
            write!(stream, "int {}() {{\n\n}}\n\n", function.identifier)?;
        }

        return Ok(())
    }
}
