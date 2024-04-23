use std::ptr::write_unaligned;

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Code {
    NOOP,
    RETURN,
    TRANSPILE_ADD,
    LOAD8,
    LOAD16,
    LOAD32,
    LOAD64,
    LOAD128,
    LOAD_LOCAL,
    STORE_LOCAL,
    AND,
    OR,
    ADD,
    SUB,
    MUL,
    DIV,
    EQ,
    NEQ,
    GR,
    GR_EQ,
    LE,
    LE_EQ,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Primitive {
    BOOL,
    I32,
    I64,
    U32,
    U64,
    F32,
    F64,
}

pub struct Chunk {
    pub code: Vec<u8>,
    pub locals: Vec<u8>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: vec![],
            locals: vec![],
        }
    }

    pub fn push(&mut self, code: Code) {
        self.code.push(code as u8)
    }

    pub fn push_with_u8(&mut self, code: Code, arg: u8) {
        unsafe { self.code.extend([code as u8, arg]) }
    }

    pub fn push_with_u16(&mut self, code: Code, arg: u16) {
        let len = self.code.len();

        unsafe {
            self.code.reserve(1 + 2);
            *self.code.as_mut_ptr().add(len) = code as u8;
            write_unaligned(self.code.as_mut_ptr().add(len + 1) as *mut u16, arg);
            self.code.set_len(len + 1 + 2);
        }
    }

    pub fn push_with_u32(&mut self, code: Code, arg: u32) {
        let len = self.code.len();

        unsafe {
            self.code.reserve(1 + 4);
            *self.code.as_mut_ptr().add(len) = code as u8;
            write_unaligned(self.code.as_mut_ptr().add(len + 1) as *mut u32, arg);
            self.code.set_len(len + 1 + 4);
        }
    }

    pub fn push_with_u64(&mut self, code: Code, arg: u64) {
        let len = self.code.len();

        unsafe {
            self.code.reserve(1 + 8);
            *self.code.as_mut_ptr().add(len) = code as u8;
            write_unaligned(self.code.as_mut_ptr().add(len + 1) as *mut u64, arg);
            self.code.set_len(len + 1 + 8);
        }
    }

    pub fn push_with_u128(&mut self, code: Code, arg: u128) {
        let len = self.code.len();

        unsafe {
            self.code.reserve(1 + 16);
            *self.code.as_mut_ptr().add(len) = code as u8;
            write_unaligned(self.code.as_mut_ptr().add(len + 1) as *mut u128, arg);
            self.code.set_len(len + 1 + 16);
        }
    }
}
