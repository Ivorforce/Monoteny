use crate::program::types::TypeProto;

#[derive(Copy, Clone)]
pub union Value {
    pub bool: bool,
    pub u8: u8,
    pub u16: u16,
    pub u32: u32,
    pub u64: u64,
    pub i8: u8,
    pub i16: u16,
    pub i32: u32,
    pub i64: u64,
    pub f32: f32,
    pub f64: f64,
    pub ptr: *mut (),
}

impl Value {
    pub fn alloc() -> Value {
        Value { u8: 0 }
    }
}

pub fn get_size_bytes(type_: &TypeProto) -> usize {
    0  // TODO
}

pub fn bytes_to_stack_slots(size: usize) -> u8 {
    0 // TODO
}
