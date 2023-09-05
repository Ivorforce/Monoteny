use std::io::Write;
use crate::program::Program;

pub fn transpile_program(
    program: &Program,
    header_stream: &mut (dyn Write),
    source_stream: &mut (dyn Write)
) -> Result<(), std::io::Error> {
    todo!()
    // writeln!(header_stream, "#include <cstdint>")?;
    // writeln!(header_stream, "#include <iostream>")?;
    // writeln!(header_stream, "#include <Eigen/Tensor>")?;
    // write!(header_stream, "\n\n")?;
    //
    // for implementation in program.module.function_implementations.values() {
    //     let return_type = transpile_type(&implementation.head.interface.return_type);
    //
    //     write!(header_stream, "{} {}(", return_type, implementation.head.name)?;
    //
    //     for parameter in implementation.parameter_variables.iter() {
    //         // External names do not exist in C. Let's just use the internal name.
    //         write!(header_stream, "{} {},", transpile_type(&parameter.type_), implementation.variable_names.get(parameter).unwrap())?;
    //     }
    //
    //     write!(header_stream, ") {{\n\n}}\n\n")?;
    // }
    //
    // return Ok(())
}
//
// pub fn transpile_primitive_type(type_def: &primitives::Type) -> String {
//     String::from(match type_def {
//         primitives::Type::Bool => "bool",
//         primitives::Type::Int8 => "int8_t",
//         primitives::Type::Int16 => "int16_t",
//         primitives::Type::Int32 => "int32_t",
//         primitives::Type::Int64 => "int64_t",
//         primitives::Type::Int128 => "int128_t",
//         primitives::Type::UInt8 => "uint8_t",
//         primitives::Type::UInt16 => "uint16_t",
//         primitives::Type::UInt32 => "uint32_t",
//         primitives::Type::UInt64 => "uint64_t",
//         primitives::Type::UInt128 => "uint128_t",
//         primitives::Type::Float32 => "float32",
//         primitives::Type::Float64 => "float64",
//     })
// }
//
// pub fn transpile_type(type_def: &TypeProto) -> String {
//     match &type_def.unit {
//         TypeUnit::Struct(t) => todo!(),
//         TypeUnit::Monad => {
//             // TODO Shape
//             format!("Tensor<{}, 1>", transpile_type(&type_def.arguments[0]))
//         }
//         TypeUnit::Generic(_) => todo!(),
//         TypeUnit::Any(_) => format!("Any"),
//         TypeUnit::MetaType => todo!(),
//         TypeUnit::Void => todo!(),
//         TypeUnit::Function(_) => todo!(),
//     }
// }