use std::ptr::write_unaligned;
use crate::interpreter::data::Value;
use crate::interpreter::opcode::OpCode;

pub struct Chunk {
    pub code: Vec<u8>,
    pub locals_count: u32,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: vec![],
            locals_count: 0,
            constants: vec![],
        }
    }

    pub fn push(&mut self, code: OpCode) {
        self.code.push(code as u8)
    }

    pub fn push_with_u8(&mut self, code: OpCode, arg: u8) {
        unsafe { self.code.extend([code as u8, arg]) }
    }

    pub fn push_with_u16(&mut self, code: OpCode, arg: u16) {
        let len = self.code.len();

        unsafe {
            self.code.reserve(1 + 2);
            *self.code.as_mut_ptr().add(len) = code as u8;
            write_unaligned(self.code.as_mut_ptr().add(len + 1) as *mut u16, arg);
            self.code.set_len(len + 1 + 2);
        }
    }

    pub fn push_with_u32(&mut self, code: OpCode, arg: u32) {
        let len = self.code.len();

        unsafe {
            self.code.reserve(1 + 4);
            *self.code.as_mut_ptr().add(len) = code as u8;
            write_unaligned(self.code.as_mut_ptr().add(len + 1) as *mut u32, arg);
            self.code.set_len(len + 1 + 4);
        }
    }

    pub fn push_with_u64(&mut self, code: OpCode, arg: u64) {
        let len = self.code.len();

        unsafe {
            self.code.reserve(1 + 8);
            *self.code.as_mut_ptr().add(len) = code as u8;
            write_unaligned(self.code.as_mut_ptr().add(len + 1) as *mut u64, arg);
            self.code.set_len(len + 1 + 8);
        }
    }

    pub fn modify_u32(&mut self, position: usize, arg: u32) {
        unsafe {
            write_unaligned(self.code.as_mut_ptr().add(position) as *mut u32, arg);
        }
    }
}
