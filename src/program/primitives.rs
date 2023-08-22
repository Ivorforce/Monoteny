use std::fmt::Debug;
use strum::EnumIter;


#[derive(Copy, Clone, PartialEq, Eq, Hash, EnumIter, Debug)]
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

impl Type {
    pub fn is_number(&self) -> bool {
        match self {
            Type::Bool => false,
            _ => true,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            Type::Float32 => true,
            Type::Float64 => true,
            _ => false,
        }
    }

    pub fn is_int(&self) -> bool {
        match self {
            Type::Bool => false,
            Type::Float32 => false,
            Type::Float64 => false,
            _ => true,
        }
    }

    pub fn is_signed_number(&self) -> bool {
        match self {
            Type::Bool => false,
            Type::UInt8 => false,
            Type::UInt16 => false,
            Type::UInt32 => false,
            Type::UInt64 => false,
            Type::UInt128 => false,
            _ => true,
        }
    }

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
