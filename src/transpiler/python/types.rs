use std::io::Write;
use crate::program::builtins::Builtins;
use crate::program::primitives;
use crate::program::structs::Struct;
use crate::program::types::{TypeProto, TypeUnit};
use crate::transpiler::python::TranspilerContext;

pub fn transpile(stream: &mut (dyn Write), type_def: &TypeProto, context: &TranspilerContext) -> Result<(), std::io::Error> {
    match &type_def.unit {
        TypeUnit::Primitive(n) => transpile_primitive(stream, n)?,
        TypeUnit::Struct(s) => transpile_struct(stream, s, context)?,
        TypeUnit::Trait(t) => todo!(),
        TypeUnit::Monad => write!(stream, "np.ndarray")?,
        TypeUnit::FunctionOverload(_) => todo!(),
        TypeUnit::Generic(_) => todo!(),
        TypeUnit::Any(_) => write!(stream, "Any")?,  // TODO Use generics instead
        TypeUnit::MetaType => todo!(),
        TypeUnit::PrecedenceGroup(_) => todo!(),
        TypeUnit::Void => todo!(),
        TypeUnit::AnonymousStruct(_) => todo!(),
        TypeUnit::Keyword(_) => todo!(),
    }

    Ok(())
}

pub fn transpile_primitive_value(stream: &mut (dyn Write), value: &primitives::Value) -> Result<(), std::io::Error> {
    Ok(match value {
        primitives::Value::Bool(n) => write!(stream, "{}", (if *n { "True" } else { "False" }))?,
        primitives::Value::Int8(n) => write!(stream, "int8({})", n)?,
        primitives::Value::Int16(n) => write!(stream, "int16({})", n)?,
        primitives::Value::Int32(n) => write!(stream, "int32({})", n)?,
        primitives::Value::Int64(n) => write!(stream, "int64({})", n)?,
        primitives::Value::Int128(n) => write!(stream, "int128({})", n)?,
        primitives::Value::UInt8(n) => write!(stream, "uint8({})", n)?,
        primitives::Value::UInt16(n) => write!(stream, "uint16({})", n)?,
        primitives::Value::UInt32(n) => write!(stream, "uint32({})", n)?,
        primitives::Value::UInt64(n) => write!(stream, "uint64({})", n)?,
        primitives::Value::UInt128(n) => write!(stream, "uint128({})", n)?,
        primitives::Value::Float32(n) => write!(stream, "float32({})", n)?,
        primitives::Value::Float64(n) => write!(stream, "float64({})", n)?,
    })
}

pub fn transpile_struct(stream: &mut (dyn Write), s: &Struct, context: &TranspilerContext) -> Result<(), std::io::Error> {
    if s == context.builtins.strings.String.as_ref() {
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
