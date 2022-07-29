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
        let return_type = function.interface.return_type.as_ref()
            .map(|x| transpile_type(&x))
            .unwrap_or_else(|| String::from("void"));

        write!(header_stream, "{} {}(", return_type, function.interface.name)?;

        for parameter in function.interface.parameters.iter() {
            // External names do not exist in C
            write!(header_stream, "{} {},", transpile_type(&parameter.variable.type_declaration), parameter.variable.name)?;
        }

        write!(header_stream, ") {{\n\n}}\n\n")?;
    }

    return Ok(())
}

pub fn transpile_primitive_type(type_def: &PrimitiveType) -> String {
    String::from(match type_def {
        PrimitiveType::Bool => "bool",
        PrimitiveType::Int8 => "int8_t",
        PrimitiveType::Int16 => "int16_t",
        PrimitiveType::Int32 => "int32_t",
        PrimitiveType::Int64 => "int64_t",
        PrimitiveType::Int128 => "int128_t",
        PrimitiveType::UInt8 => "uint8_t",
        PrimitiveType::UInt16 => "uint16_t",
        PrimitiveType::UInt32 => "uint32_t",
        PrimitiveType::UInt64 => "uint64_t",
        PrimitiveType::UInt128 => "uint128_t",
        PrimitiveType::Float32 => "float",
        PrimitiveType::Float64 => "double",
    })
}

pub fn transpile_type(type_def: &Type) -> String {
    match type_def.borrow() {
        Type::Primitive(n) => transpile_primitive_type(n),
        Type::Identifier(t) => {
            match t.as_str() {
                "String" => todo!(),
                _ => t.clone()
            }
        },
        Type::NDArray(atom) => {
            // TODO Shape
            format!("Tensor<{}, 1>", transpile_type(atom))
        }
        Type::Function(_) => todo!(),
        Type::Generic(_) => todo!(),
    }
}