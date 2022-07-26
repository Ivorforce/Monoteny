use std::borrow::Borrow;
use std::io::Write;
use crate::computation_tree;
use crate::computation_tree::{Program, Type};

pub struct CPPTranspiler {

}

impl CPPTranspiler {
    pub fn transpile(
        &self,
        program: &Program,
        header_stream: &mut (dyn Write),
        source_stream: &mut (dyn Write)
    ) -> Result<(), std::io::Error> {
        writeln!(header_stream, "#include <cstdint>")?;
        writeln!(header_stream, "#include <iostream>")?;
        writeln!(header_stream, "#include <Eigen/Tensor>")?;
        write!(header_stream, "\n\n")?;

        for function in program.functions.iter() {
            let return_type = function.return_type.as_ref()
                .map(|x| self.transpile_type(&x))
                .unwrap_or_else(|| String::from("void"));

            write!(header_stream, "{} {}() {{\n\n}}\n\n", return_type, function.identifier)?;
        }

        return Ok(())
    }

    pub fn transpile_type(&self, type_def: &Type) -> String {
        match type_def.borrow() {
            Type::Identifier(t) => {
                match t.as_str() {
                    "Int32" => String::from("int32_t"),
                    "Int64" => String::from("int64_t"),
                    "Float32" => String::from("float_t"),
                    "Float64" => String::from("float64_t"),
                    _ => t.clone()
                }
            },
            Type::NDArray(atom) => {
                // TODO Shape
                format!("Tensor<{}, 1>", self.transpile_type(atom))
            }
        }
    }
}
