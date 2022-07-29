use crate::abstract_syntax;

#[derive(Copy, Clone, PartialEq)]
pub enum PrimitiveValue {
    Bool(bool),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Int128(i128),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    UInt128(u128),
    Float32(f32),
    Float64(f64),
}

#[derive(Copy, Clone, PartialEq)]
pub enum PrimitiveType {
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    Float32,
    Float64,
}

impl PrimitiveValue {
    pub fn primitive_type(&self) -> PrimitiveType {
        use PrimitiveValue::*;
        match self {
            Bool(_) => PrimitiveType::Bool,
            Int8(_) => PrimitiveType::Int8,
            Int16(_) => PrimitiveType::Int16,
            Int32(_) => PrimitiveType::Int32,
            Int64(_) => PrimitiveType::Int64,
            Int128(_) => PrimitiveType::Int128,
            UInt8(_) => PrimitiveType::UInt8,
            UInt16(_) => PrimitiveType::UInt16,
            UInt32(_) => PrimitiveType::UInt32,
            UInt64(_) => PrimitiveType::UInt64,
            UInt128(_) => PrimitiveType::UInt128,
            Float32(_) => PrimitiveType::Float32,
            Float64(_) => PrimitiveType::Float64,
        }
    }
}

impl PrimitiveType {
    pub fn identifier_string(&self) -> String {
        use PrimitiveType::*;
        String::from(match self {
            Bool => "Bool",
            Int8 => "Int8",
            Int16 => "Int16",
            Int32 => "Int32",
            Int64 => "Int64",
            Int128 => "Int128",
            UInt8 => "UInt8",
            UInt16 => "UInt16",
            UInt32 => "UInt32",
            UInt64 => "UInt64",
            UInt128 => "UInt128",
            Float32 => "Float32",
            Float64 => "Float64",
        })
    }
}

pub fn resolve_primitive_type(identifier: &str) -> Option<PrimitiveType> {
    use PrimitiveType::*;
    
    match identifier {
        "Bool" => Some(Bool),
        "Int8" => Some(Int8),
        "Int16" => Some(Int16),
        "Int32" => Some(Int32),
        "Int64" => Some(Int64),
        "Int128" => Some(Int128),
        "UInt8" => Some(UInt8),
        "UInt16" => Some(UInt16),
        "UInt32" => Some(UInt32),
        "UInt64" => Some(UInt64),
        "UInt128" => Some(UInt128),
        "Float32" => Some(Float32),
        "Float64" => Some(Float64),
        _ => None
    }
}
