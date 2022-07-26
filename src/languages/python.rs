use std::borrow::Borrow;
use std::io::Write;
use crate::computation_tree;
use crate::computation_tree::{Program, Type};

pub struct PythonTranspiler {

}

impl PythonTranspiler {
    pub fn transpile(&self, program: &Program, stream: &mut (dyn Write)) -> Result<(), std::io::Error> {
        write!(stream, "import numpy as np\n\n")?;

        for function in program.functions.iter() {
            write!(stream, "def {}()", function.identifier)?;
            match &function.return_type {
                Some(t) => write!(stream, " -> {}", self.transpile_type(&t))?,
                _ => {}
            }
            write!(stream, ":\n    pass\n\n")?;

        }

        return Ok(())
    }

    pub fn transpile_type(&self, type_def: &Type) -> String {
        match type_def.borrow() {
            Type::Identifier(t) => {
                match t.as_str() {
                    "Int32" => String::from("np.int32"),
                    _ => t.clone()
                }
            },
            Type::NDArray(_) => {
                format!("np.ndarray")
            }
        }
    }
}
