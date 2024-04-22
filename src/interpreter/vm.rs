use std::mem::transmute;
use monoteny_macro::{bin_op, bool_bin_op, pop_ip, to_bool_bin_op};
use std::ptr::{write_unaligned, read_unaligned};
use crate::error::{RResult, RuntimeError};
use crate::interpreter::chunks::{Chunk, Code, Primitive};
use crate::interpreter::disassembler::disassemble_one;

pub struct VM<'a> {
    pub chunk: &'a Chunk,
    pub stack: Vec<u32>,
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
            let mut sp: *mut u32 = &mut self.stack[0] as *mut u32;

            loop {
                disassemble_one(ip);
                print!("\n");

                let code = transmute::<u8, Code>(*ip);
                ip = ip.add(1);

                match code {
                    Code::NOOP => {},
                    Code::RETURN => return Ok(()),
                    Code::LOAD8 => {
                        *sp = 0;
                        *(sp as *mut u8) = pop_ip!(u8);
                        sp = sp.add(1);
                    },
                    Code::LOAD16 => {
                        *sp = 0;
                        *(sp as *mut u16) = pop_ip!(u16);
                        sp = sp.add(1);
                    },
                    Code::LOAD32 => {
                        *sp = pop_ip!(u32);
                        sp = sp.add(1);
                    },
                    Code::LOAD64 => {
                        *sp = 0;
                        *(sp as *mut u64) = pop_ip!(u64);
                        sp = sp.add(2);
                    },
                    Code::AND => bool_bin_op!(&&),
                    Code::OR => bool_bin_op!(||),
                    Code::ADD => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U32 => bin_op!(u32 +),
                            Primitive::U64 => bin_op!(u64 +),
                            Primitive::I32 => bin_op!(i32 +),
                            Primitive::I64 => bin_op!(i64 +),
                            Primitive::F32 => bin_op!(f32 +),
                            Primitive::F64 => bin_op!(f64 +),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    Code::SUB => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U32 => bin_op!(u32 -),
                            Primitive::U64 => bin_op!(u64 -),
                            Primitive::I32 => bin_op!(i32 -),
                            Primitive::I64 => bin_op!(i64 -),
                            Primitive::F32 => bin_op!(f32 -),
                            Primitive::F64 => bin_op!(f64 -),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    Code::MUL => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U32 => bin_op!(u32 *),
                            Primitive::U64 => bin_op!(u64 *),
                            Primitive::I32 => bin_op!(i32 *),
                            Primitive::I64 => bin_op!(i64 *),
                            Primitive::F32 => bin_op!(f32 *),
                            Primitive::F64 => bin_op!(f64 *),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    Code::DIV => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U32 => bin_op!(u32 /),
                            Primitive::U64 => bin_op!(u64 /),
                            Primitive::I32 => bin_op!(i32 /),
                            Primitive::I64 => bin_op!(i64 /),
                            Primitive::F32 => bin_op!(f32 /),
                            Primitive::F64 => bin_op!(f64 /),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    Code::EQ => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::BOOL => bool_bin_op!(==),
                            Primitive::U32 => to_bool_bin_op!(u32 ==),
                            Primitive::U64 => to_bool_bin_op!(u64 ==),
                            Primitive::I32 => to_bool_bin_op!(i32 ==),
                            Primitive::I64 => to_bool_bin_op!(i64 ==),
                            Primitive::F32 => to_bool_bin_op!(f32 ==),
                            Primitive::F64 => to_bool_bin_op!(f64 ==),
                        }
                    },
                    Code::NEQ => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::BOOL => bool_bin_op!(==),
                            Primitive::U32 => to_bool_bin_op!(u32 !=),
                            Primitive::U64 => to_bool_bin_op!(u64 !=),
                            Primitive::I32 => to_bool_bin_op!(i32 !=),
                            Primitive::I64 => to_bool_bin_op!(i64 !=),
                            Primitive::F32 => to_bool_bin_op!(f32 !=),
                            Primitive::F64 => to_bool_bin_op!(f64 !=),
                        }
                    },
                    Code::GR => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U32 => to_bool_bin_op!(u32 >),
                            Primitive::U64 => to_bool_bin_op!(u64 >),
                            Primitive::I32 => to_bool_bin_op!(i32 >),
                            Primitive::I64 => to_bool_bin_op!(i64 >),
                            Primitive::F32 => to_bool_bin_op!(f32 >),
                            Primitive::F64 => to_bool_bin_op!(f64 >),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    Code::GR_EQ => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U32 => to_bool_bin_op!(u32 >=),
                            Primitive::U64 => to_bool_bin_op!(u64 >=),
                            Primitive::I32 => to_bool_bin_op!(i32 >=),
                            Primitive::I64 => to_bool_bin_op!(i64 >=),
                            Primitive::F32 => to_bool_bin_op!(f32 >=),
                            Primitive::F64 => to_bool_bin_op!(f64 >=),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    Code::LE => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U32 => to_bool_bin_op!(u32 <),
                            Primitive::U64 => to_bool_bin_op!(u64 <),
                            Primitive::I32 => to_bool_bin_op!(i32 <),
                            Primitive::I64 => to_bool_bin_op!(i64 <),
                            Primitive::F32 => to_bool_bin_op!(f32 <),
                            Primitive::F64 => to_bool_bin_op!(f64 <),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    Code::LE_EQ => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U32 => to_bool_bin_op!(u32 <=),
                            Primitive::U64 => to_bool_bin_op!(u64 <=),
                            Primitive::I32 => to_bool_bin_op!(i32 <=),
                            Primitive::I64 => to_bool_bin_op!(i64 <=),
                            Primitive::F32 => to_bool_bin_op!(f32 <=),
                            Primitive::F64 => to_bool_bin_op!(f64 <=),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                }
            }
        }
    }
}
