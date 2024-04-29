use std::alloc::{alloc, Layout};
use std::intrinsics::transmute;
use std::ptr::write_unaligned;
use crate::program::types::TypeProto;

#[derive(Copy, Clone)]
pub union Value {
    pub bool: bool,
    pub u8: u8,
    pub u16: u16,
    pub u32: u32,
    pub u64: u64,
    pub i8: i8,
    pub i16: i16,
    pub i32: i32,
    pub i64: i64,
    pub f32: f32,
    pub f64: f64,
    pub ptr: *mut (),
}

impl Value {
    pub fn alloc() -> Value {
        Value { u8: 0 }
    }
}

pub unsafe fn string_to_ptr(string: &String) -> *mut () {
    let data = alloc(Layout::new::<String>());
    write_unaligned(data as *mut String, string.clone());
    transmute(data)
}
