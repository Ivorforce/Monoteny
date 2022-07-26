use std::io::Write;
use crate::computation_tree;
use crate::computation_tree::Program;

use crate::languages::transpiler::Transpiler;

pub struct PythonTranspiler {

}

impl Transpiler for PythonTranspiler {
    fn transpile(&self, program: Program, stream: &mut (dyn Write)) -> Result<(), std::io::Error> {
        println!("{}", program.functions.len());

        for function in program.functions {
            write!(stream, "def {}():\n    pass\n\n", function.identifier)?;
        }

        return Ok(())
    }
}
