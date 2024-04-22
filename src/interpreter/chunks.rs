use std::mem::transmute;

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Code {
    NOOP,
    ADD,
    SUB,
    MUL,
    DIV,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Primitive {
    BOOL,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
}

pub struct Chunk {
    pub code: Vec<Code>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: vec![],
        }
    }

    pub fn push(&mut self, code: Code) {
        self.code.push(code)
    }

    pub fn push2(&mut self, code: Code, arg: u8) {
        unsafe { self.code.extend([code, transmute::<u8, Code>(arg)]) }
    }
}
