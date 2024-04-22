use std::mem::transmute;
use monoteny_macro::{bin_op, load_ip};
use std::ptr::{write_unaligned, read_unaligned};
use crate::error::{RResult, RuntimeError};
use crate::interpreter::chunks::{Chunk, Code, Primitive};
use crate::interpreter::disassembler::disassemble_one;

pub struct VM<'a> {
    pub chunk: &'a Chunk,
    pub stack: Vec<u8>,
}

impl<'a> VM<'a> {
    pub fn new(chunk: &'a Chunk) -> VM {
        VM {
            chunk,
            stack: vec![0; 1024],
        }
    }

    pub fn run(&mut self) -> RResult<()> {
        unsafe {
            let mut ip: *const u8 = transmute(&self.chunk.code[0]);
            let mut sp: *mut u8 = &mut self.stack[0] as *mut u8;

            loop {
                disassemble_one(ip);
                print!("\n");

                match transmute::<u8, Code>(*ip) {
                    Code::NOOP => ip = ip.add(1),
                    Code::RETURN => return Ok(()),
                    Code::LOAD8 => load_ip!(ip sp u8),
                    Code::LOAD16 => load_ip!(ip sp u16),
                    Code::LOAD32 => load_ip!(ip sp u32),
                    Code::LOAD64 => load_ip!(ip sp u64),
                    Code::LOAD128 => load_ip!(ip sp u128),
                    Code::ADD => {
                        let arg: Primitive = transmute(*ip.add(1));
                        ip = ip.add(2);

                        match arg {
                            Primitive::BOOL => return Err(RuntimeError::new("Cannot add bool.".to_string())),
                            Primitive::U8 => bin_op!(sp u8 +),
                            Primitive::U16 => bin_op!(sp u16 +),
                            Primitive::U32 => bin_op!(sp u32 +),
                            Primitive::U64 => bin_op!(sp u64 +),
                            Primitive::U128 => bin_op!(sp u128 +),
                            Primitive::I8 => bin_op!(sp i8 +),
                            Primitive::I16 => bin_op!(sp i16 +),
                            Primitive::I32 => bin_op!(sp i32 +),
                            Primitive::I64 => bin_op!(sp i64 +),
                            Primitive::I128 => bin_op!(sp i128 +),
                            Primitive::F32 => bin_op!(sp f32 +),
                            Primitive::F64 => bin_op!(sp f64 +),
                        }
                    },
                    Code::SUB => {
                        let arg: Primitive = transmute(*ip.add(1));
                        ip = ip.add(2);

                        match arg {
                            Primitive::BOOL => return Err(RuntimeError::new("Cannot add bool.".to_string())),
                            Primitive::U8 => bin_op!(sp u8 -),
                            Primitive::U16 => bin_op!(sp u16 -),
                            Primitive::U32 => bin_op!(sp u32 -),
                            Primitive::U64 => bin_op!(sp u64 -),
                            Primitive::U128 => bin_op!(sp u128 -),
                            Primitive::I8 => bin_op!(sp i8 -),
                            Primitive::I16 => bin_op!(sp i16 -),
                            Primitive::I32 => bin_op!(sp i32 -),
                            Primitive::I64 => bin_op!(sp i64 -),
                            Primitive::I128 => bin_op!(sp i128 -),
                            Primitive::F32 => bin_op!(sp f32 -),
                            Primitive::F64 => bin_op!(sp f64 -),
                        }
                    },
                    Code::MUL => {
                        let arg: Primitive = transmute(*ip.add(1));
                        ip = ip.add(2);

                        match arg {
                            Primitive::BOOL => return Err(RuntimeError::new("Cannot add bool.".to_string())),
                            Primitive::U8 => bin_op!(sp u8 *),
                            Primitive::U16 => bin_op!(sp u16 *),
                            Primitive::U32 => bin_op!(sp u32 *),
                            Primitive::U64 => bin_op!(sp u64 *),
                            Primitive::U128 => bin_op!(sp u128 *),
                            Primitive::I8 => bin_op!(sp i8 *),
                            Primitive::I16 => bin_op!(sp i16 *),
                            Primitive::I32 => bin_op!(sp i32 *),
                            Primitive::I64 => bin_op!(sp i64 *),
                            Primitive::I128 => bin_op!(sp i128 *),
                            Primitive::F32 => bin_op!(sp f32 *),
                            Primitive::F64 => bin_op!(sp f64 *),
                        }
                    },
                    Code::DIV => {
                        let arg: Primitive = transmute(*ip.add(1));
                        ip = ip.add(2);

                        match arg {
                            Primitive::BOOL => return Err(RuntimeError::new("Cannot add bool.".to_string())),
                            Primitive::U8 => bin_op!(sp u8 /),
                            Primitive::U16 => bin_op!(sp u16 /),
                            Primitive::U32 => bin_op!(sp u32 /),
                            Primitive::U64 => bin_op!(sp u64 /),
                            Primitive::U128 => bin_op!(sp u128 /),
                            Primitive::I8 => bin_op!(sp i8 /),
                            Primitive::I16 => bin_op!(sp i16 /),
                            Primitive::I32 => bin_op!(sp i32 /),
                            Primitive::I64 => bin_op!(sp i64 /),
                            Primitive::I128 => bin_op!(sp i128 /),
                            Primitive::F32 => bin_op!(sp f32 /),
                            Primitive::F64 => bin_op!(sp f64 /),
                        }
                    },
                }
            }
        }
    }
}
