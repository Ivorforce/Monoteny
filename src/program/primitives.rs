use std::fmt::Debug;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Type {
    Bool,
    Int(usize),
    UInt(usize),
    Float(usize),
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
            Type::Float(_) => true,
            _ => false,
        }
    }

    pub fn is_int(&self) -> bool {
        match self {
            Type::Bool => false,
            Type::Float(_) => false,
            _ => true,
        }
    }

    pub fn is_signed_number(&self) -> bool {
        match self {
            Type::Bool => false,
            Type::UInt(_) => false,
            _ => true,
        }
    }

    pub fn identifier_string(&self) -> String {
        match self {
            Type::Bool => "Bool".to_string(),
            Type::Int(bits) => format!("Int{}", bits),
            Type::UInt(bits) => format!("UInt{}", bits),
            Type::Float(bits) => format!("Float{}", bits),
        }
    }
}
