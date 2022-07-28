use std::borrow::Borrow;
use std::io::Write;
use crate::linker::computation_tree::*;

pub fn transpile_program(
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
            .map(|x| transpile_type(&x))
            .unwrap_or_else(|| String::from("void"));

        write!(header_stream, "{} {}(", return_type, function.name)?;

        for parameter in function.parameters.iter() {
            // External names do not exist in C
            write!(header_stream, "{} {},", transpile_type(&parameter.variable.type_declaration), parameter.variable.name)?;
        }

        write!(header_stream, ") {{\n\n}}\n\n")?;
    }

    return Ok(())
}

pub fn transpile_type(type_def: &Type) -> String {
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
            format!("Tensor<{}, 1>", transpile_type(atom))
        }
        Type::Function(_) => todo!(),
        Type::Generic(_) => todo!()
    }
}