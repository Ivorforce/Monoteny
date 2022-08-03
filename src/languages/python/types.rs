use std::io::Write;
use crate::linker::computation_tree::{Struct, Type};
use crate::linker::primitives;

pub fn transpile(stream: &mut (dyn Write), type_def: &Type) -> Result<(), std::io::Error> {
    match type_def {
        Type::Primitive(n) => transpile_primitive(stream, n)?,
        Type::Struct(s) => transpile_struct(stream, s)?,
        Type::NDArray(_) => write!(stream, "np.ndarray")?,
        Type::Function(_) => todo!(),
        Type::Generic(_) => todo!(),
        Type::MetaType(_) => todo!()
    }

    Ok(())
}

pub fn transpile_primitive_value(stream: &mut (dyn Write), value: &primitives::Value) -> Result<(), std::io::Error> {
    Ok(match value {
        primitives::Value::Bool(n) => write!(stream, "{}", (if *n { "True" } else { "False" }))?,
        primitives::Value::Int8(n) => write!(stream, "np.int8({})", n)?,
        primitives::Value::Int16(n) => write!(stream, "np.int16({})", n)?,
        primitives::Value::Int32(n) => write!(stream, "np.int32({})", n)?,
        primitives::Value::Int64(n) => write!(stream, "np.int64({})", n)?,
        primitives::Value::Int128(n) => write!(stream, "np.int128({})", n)?,
        primitives::Value::UInt8(n) => write!(stream, "np.uint8({})", n)?,
        primitives::Value::UInt16(n) => write!(stream, "np.uint16({})", n)?,
        primitives::Value::UInt32(n) => write!(stream, "np.uint32({})", n)?,
        primitives::Value::UInt64(n) => write!(stream, "np.uint64({})", n)?,
        primitives::Value::UInt128(n) => write!(stream, "np.uint128({})", n)?,
        primitives::Value::Float32(n) => write!(stream, "np.float32({})", n)?,
        primitives::Value::Float64(n) => write!(stream, "np.float64({})", n)?,
    })
}

pub fn transpile_struct(stream: &mut (dyn Write), type_def: &Struct) -> Result<(), std::io::Error> {
    // TODO We should use builtin references to check for struct identity, not just the name
    write!(stream, "{}", match type_def.name.as_str() {
        "String" => "str",
        _ => panic!("Unknown struct: {}", type_def.name)
    })?;

    Ok(())
}

pub fn transpile_primitive(stream: &mut (dyn Write), type_def: &primitives::Type) -> Result<(), std::io::Error> {
    use primitives::Type::*;
    match type_def {
        Bool => write!(stream, "np.bool")?,
        Int8 => write!(stream, "np.int8")?,
        Int16 => write!(stream, "np.int16")?,
        Int32 => write!(stream, "np.int32")?,
        Int64 => write!(stream, "np.int64")?,
        Int128 => write!(stream, "np.int128")?,
        UInt8 => write!(stream, "np.uint8")?,
        UInt16 => write!(stream, "np.uint16")?,
        UInt32 => write!(stream, "np.uint32")?,
        UInt64 => write!(stream, "np.uint64")?,
        UInt128 => write!(stream, "np.uint128")?,
        Float32 => write!(stream, "np.float32")?,
        Float64 => write!(stream, "np.float64")?,
    }

    Ok(())
}
