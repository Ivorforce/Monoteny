use std::io::Write;
use crate::program::primitives;
use crate::program::traits::Trait;
use crate::program::types::{TypeProto, TypeUnit};
use crate::transpiler::python::FunctionContext;

pub fn transpile(stream: &mut (dyn Write), type_def: &TypeProto, context: &FunctionContext) -> Result<(), std::io::Error> {
    match &type_def.unit {
        TypeUnit::Struct(s) => transpile_struct(stream, s, context)?,
        TypeUnit::Monad => write!(stream, "np.ndarray")?,
        TypeUnit::Generic(_) => todo!(),
        TypeUnit::Any(_) => todo!(),
        TypeUnit::MetaType => todo!(),
        TypeUnit::Void => todo!(),
        TypeUnit::Function(_) => todo!(),
    }

    Ok(())
}

pub fn transpile_primitive_value(stream: &mut (dyn Write), value: &String, type_: &primitives::Type) -> Result<(), std::io::Error> {
    Ok(match type_ {
        primitives::Type::Int8 => write!(stream, "int8({})", value)?,
        primitives::Type::Int16 => write!(stream, "int16({})", value)?,
        primitives::Type::Int32 => write!(stream, "int32({})", value)?,
        primitives::Type::Int64 => write!(stream, "int64({})", value)?,
        primitives::Type::Int128 => write!(stream, "int128({})", value)?,
        primitives::Type::UInt8 => write!(stream, "uint8({})", value)?,
        primitives::Type::UInt16 => write!(stream, "uint16({})", value)?,
        primitives::Type::UInt32 => write!(stream, "uint32({})", value)?,
        primitives::Type::UInt64 => write!(stream, "uint64({})", value)?,
        primitives::Type::UInt128 => write!(stream, "uint128({})", value)?,
        primitives::Type::Float32 => write!(stream, "float32({})", value)?,
        primitives::Type::Float64 => write!(stream, "float64({})", value)?,
        _ => panic!(),
    })
}

pub fn transpile_struct(stream: &mut (dyn Write), s: &Trait, context: &FunctionContext) -> Result<(), std::io::Error> {
    if let Some(primitive_type) = context.builtins.core.get_primitive(s) {
        transpile_primitive(stream, primitive_type)
    }
    else if s == context.builtins.core.traits.String.as_ref() {
        write!(stream, "str")
    }
    else {
        write!(stream, "{}", s.name)
    }
}

pub fn transpile_primitive(stream: &mut (dyn Write), type_def: &primitives::Type) -> Result<(), std::io::Error> {
    use crate::program::primitives::Type::*;
    match type_def {
        Bool => write!(stream, "bool")?,
        Int8 => write!(stream, "int8")?,
        Int16 => write!(stream, "int16")?,
        Int32 => write!(stream, "int32")?,
        Int64 => write!(stream, "int64")?,
        Int128 => write!(stream, "int128")?,
        UInt8 => write!(stream, "uint8")?,
        UInt16 => write!(stream, "uint16")?,
        UInt32 => write!(stream, "uint32")?,
        UInt64 => write!(stream, "uint64")?,
        UInt128 => write!(stream, "uint128")?,
        Float32 => write!(stream, "float32")?,
        Float64 => write!(stream, "float64")?,
    }

    Ok(())
}
