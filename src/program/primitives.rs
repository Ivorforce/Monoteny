use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use strum::EnumIter;


#[derive(Copy, Clone, PartialEq, Debug)]
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

    pub fn add(lhs: &Value, rhs: &Value) -> Option<Value> {
        use Value::*;

        Some(match (lhs, rhs) {
            (Int8(l), Int8(r)) => Int8(l + r),
            (Int16(l), Int16(r)) => Int16(l + r),
            (Int32(l), Int32(r)) => Int32(l + r),
            (Int64(l), Int64(r)) => Int64(l + r),
            (Int128(l), Int128(r)) => Int128(l + r),
            (UInt8(l), UInt8(r)) => UInt8(l + r),
            (UInt16(l), UInt16(r)) => UInt16(l + r),
            (UInt32(l), UInt32(r)) => UInt32(l + r),
            (UInt64(l), UInt64(r)) => UInt64(l + r),
            (UInt128(l), UInt128(r)) => UInt128(l + r),
            (Float32(l), Float32(r)) => Float32(l + r),
            (Float64(l), Float64(r)) => Float64(l + r),
            _ => return None
        })
    }

    pub fn subtract(lhs: &Value, rhs: &Value) -> Option<Value> {
        use Value::*;

        Some(match (lhs, rhs) {
            (Int8(l), Int8(r)) => Int8(l - r),
            (Int16(l), Int16(r)) => Int16(l - r),
            (Int32(l), Int32(r)) => Int32(l - r),
            (Int64(l), Int64(r)) => Int64(l - r),
            (Int128(l), Int128(r)) => Int128(l - r),
            (UInt8(l), UInt8(r)) => UInt8(l - r),
            (UInt16(l), UInt16(r)) => UInt16(l - r),
            (UInt32(l), UInt32(r)) => UInt32(l - r),
            (UInt64(l), UInt64(r)) => UInt64(l - r),
            (UInt128(l), UInt128(r)) => UInt128(l - r),
            (Float32(l), Float32(r)) => Float32(l - r),
            (Float64(l), Float64(r)) => Float64(l - r),
            _ => return None
        })
    }

    pub fn multiply(lhs: &Value, rhs: &Value) -> Option<Value> {
        use Value::*;

        Some(match (lhs, rhs) {
            (Int8(l), Int8(r)) => Int8(l * r),
            (Int16(l), Int16(r)) => Int16(l * r),
            (Int32(l), Int32(r)) => Int32(l * r),
            (Int64(l), Int64(r)) => Int64(l * r),
            (Int128(l), Int128(r)) => Int128(l * r),
            (UInt8(l), UInt8(r)) => UInt8(l * r),
            (UInt16(l), UInt16(r)) => UInt16(l * r),
            (UInt32(l), UInt32(r)) => UInt32(l * r),
            (UInt64(l), UInt64(r)) => UInt64(l * r),
            (UInt128(l), UInt128(r)) => UInt128(l * r),
            (Float32(l), Float32(r)) => Float32(l * r),
            (Float64(l), Float64(r)) => Float64(l * r),
            _ => return None
        })
    }

    pub fn divide(lhs: &Value, rhs: &Value) -> Option<Value> {
        use Value::*;

        Some(match (lhs, rhs) {
            (Int8(l), Int8(r)) => Int8(l / r),
            (Int16(l), Int16(r)) => Int16(l / r),
            (Int32(l), Int32(r)) => Int32(l / r),
            (Int64(l), Int64(r)) => Int64(l / r),
            (Int128(l), Int128(r)) => Int128(l / r),
            (UInt8(l), UInt8(r)) => UInt8(l / r),
            (UInt16(l), UInt16(r)) => UInt16(l / r),
            (UInt32(l), UInt32(r)) => UInt32(l / r),
            (UInt64(l), UInt64(r)) => UInt64(l / r),
            (UInt128(l), UInt128(r)) => UInt128(l / r),
            (Float32(l), Float32(r)) => Float32(l / r),
            (Float64(l), Float64(r)) => Float64(l / r),
            _ => return None
        })
    }
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

    pub fn parse_value(&self, value: &String) -> Option<Value> {
        match self {
            Type::Bool => None,
            Type::Int8 => i8::from_str(value.as_str()).ok().map(Value::Int8),
            Type::Int16 => i16::from_str(value.as_str()).ok().map(Value::Int16),
            Type::Int32 => i32::from_str(value.as_str()).ok().map(Value::Int32),
            Type::Int64 => i64::from_str(value.as_str()).ok().map(Value::Int64),
            Type::Int128 => i128::from_str(value.as_str()).ok().map(Value::Int128),
            Type::UInt8 => u8::from_str(value.as_str()).ok().map(Value::UInt8),
            Type::UInt16 => u16::from_str(value.as_str()).ok().map(Value::UInt16),
            Type::UInt32 => u32::from_str(value.as_str()).ok().map(Value::UInt32),
            Type::UInt64 => u64::from_str(value.as_str()).ok().map(Value::UInt64),
            Type::UInt128 => u128::from_str(value.as_str()).ok().map(Value::UInt128),
            Type::Float32 => f32::from_str(value.as_str()).ok().map(Value::Float32),
            Type::Float64 => f64::from_str(value.as_str()).ok().map(Value::Float64),
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
