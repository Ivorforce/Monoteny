#[derive(Copy, Clone, PartialEq)]
pub enum Value {
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
pub enum Type {
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

impl Value {
    pub fn get_type(&self) -> Type {
        use Value::*;
        match self {
            Bool(_) => Type::Bool,
            Int8(_) => Type::Int8,
            Int16(_) => Type::Int16,
            Int32(_) => Type::Int32,
            Int64(_) => Type::Int64,
            Int128(_) => Type::Int128,
            UInt8(_) => Type::UInt8,
            UInt16(_) => Type::UInt16,
            UInt32(_) => Type::UInt32,
            UInt64(_) => Type::UInt64,
            UInt128(_) => Type::UInt128,
            Float32(_) => Type::Float32,
            Float64(_) => Type::Float64,
        }
    }
}

impl Type {
    pub fn identifier_string(&self) -> String {
        use Type::*;
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

pub fn parse(identifier: &str) -> Option<Type> {
    use Type::*;
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
