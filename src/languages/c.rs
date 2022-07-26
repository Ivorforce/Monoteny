use std::borrow::Borrow;
use std::io::Write;
use crate::computation_tree;
use crate::computation_tree::{Program, Type};

use crate::languages::transpiler::Transpiler;

pub struct CTranspiler {

}

impl CTranspiler {
    fn transpile_type(&self, type_def: &Type) -> String {
        match type_def.borrow() {
            Type::Identifier(t) => {
                match t.as_str() {
                    "Int32" => String::from("int32_t"),
                    _ => t.clone()
                }
            },
            Type::NDArray(t) => {
                // TODO Should be a custom ndarray type
                format!("{}*", self.transpile_type(t))
            }
        }
    }
}

impl Transpiler for CTranspiler {
    fn transpile(&self, program: &Program, stream: &mut (dyn Write)) -> Result<(), std::io::Error> {
        write!(stream, "#include <stdint.h>\n\n")?;

        for function in program.functions.iter() {
            let return_type = function.return_type.as_ref()
                .map(|x| self.transpile_type(&x))
                .unwrap_or_else(|| String::from("void"));

            write!(stream, "{} {}() {{\n\n}}\n\n", return_type, function.identifier)?;
        }

        return Ok(())
    }
}
