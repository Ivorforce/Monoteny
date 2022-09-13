use std::io::Write;
use crate::linker::computation_tree::*;
use crate::program::primitives;
use crate::program::types::{Type, TypeUnit};

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

        write!(header_stream, "{} {}(", return_type, function.interface.alphanumeric_name)?;

        for parameter in function.interface.parameters.iter() {
            // External names do not exist in C
            write!(header_stream, "{} {},", transpile_type(&parameter.variable.type_declaration), parameter.variable.name)?;
        }

        write!(header_stream, ") {{\n\n}}\n\n")?;
    }

    return Ok(())
}

pub fn transpile_primitive_type(type_def: &primitives::Type) -> String {
    String::from(match type_def {
        primitives::Type::Bool => "bool",
        primitives::Type::Int8 => "int8_t",
        primitives::Type::Int16 => "int16_t",
        primitives::Type::Int32 => "int32_t",
        primitives::Type::Int64 => "int64_t",
        primitives::Type::Int128 => "int128_t",
        primitives::Type::UInt8 => "uint8_t",
        primitives::Type::UInt16 => "uint16_t",
        primitives::Type::UInt32 => "uint32_t",
        primitives::Type::UInt64 => "uint64_t",
        primitives::Type::UInt128 => "uint128_t",
        primitives::Type::Float32 => "float",
        primitives::Type::Float64 => "double",
    })
}

pub fn transpile_type(type_def: &Type) -> String {
    match &type_def.unit {
        TypeUnit::Primitive(n) => transpile_primitive_type(n),
        TypeUnit::Struct(t) => todo!(),
        TypeUnit::Monad => {
            // TODO Shape
            format!("Tensor<{}, 1>", transpile_type(&type_def.arguments[0]))
        }
        TypeUnit::Function(_) => todo!(),
        TypeUnit::Generic(_) => todo!(),
        TypeUnit::MetaType => todo!(),
        TypeUnit::PrecedenceGroup(_) => todo!()
    }
}