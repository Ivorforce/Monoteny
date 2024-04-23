use std::mem::transmute;
use monoteny_macro::{bin_op, bool_bin_op, pop_ip, pop_sp, to_bool_bin_op};
use std::ptr::{write_unaligned, read_unaligned};
use itertools::Itertools;
use uuid::Uuid;
use crate::error::{RResult, RuntimeError};
use crate::interpreter::chunks::{Chunk, OpCode, Primitive};
use crate::interpreter::data::Value;

pub struct VM<'a> {
    pub chunk: &'a Chunk,
    pub stack: Vec<Value>,
    pub locals: Vec<Value>,
    pub transpile_functions: Vec<Uuid>,
}

impl<'a> VM<'a> {
    pub fn new(chunk: &'a Chunk) -> VM {
        VM {
            chunk,
            stack: vec![Value::alloc(); 1024],
            locals: vec![Value::alloc(); usize::try_from(chunk.locals_count).unwrap()],
            transpile_functions: vec![],
        }
    }

    pub fn run(&mut self) -> RResult<()> {
        unsafe {
            let mut ip: *const u8 = transmute(&self.chunk.code[0]);
            let mut sp: *mut Value = &mut self.stack[0] as *mut Value;

            loop {
                // println!("sp: {:?}; ip: {:?}", sp, ip);
                // disassemble_one(ip);
                // print!("\n");

                let code = transmute::<u8, OpCode>(*ip);
                ip = ip.add(1);

                match code {
                    OpCode::NOOP => {},
                    OpCode::RETURN => return Ok(()),
                    OpCode::LOAD8 => {
                        (*sp).u8 = pop_ip!(u8);
                        sp = sp.add(8);
                    },
                    OpCode::LOAD16 => {
                        (*sp).u16 = pop_ip!(u16);
                        sp = sp.add(8);
                    },
                    OpCode::LOAD32 => {
                        (*sp).u32 = pop_ip!(u32);
                        sp = sp.add(8);
                    },
                    OpCode::LOAD64 => {
                        (*sp).u64 = pop_ip!(u64);
                        sp = sp.add(8);
                    },
                    OpCode::LOAD128 => {
                        let v = pop_ip!(u128);

                        (*sp).u64 = (v >> 64) as u64;
                        sp = sp.add(8);

                        (*sp).u64 = v as u64;
                        sp = sp.add(8);
                    },
                    OpCode::LOAD_LOCAL => {
                        let local_idx: u32 = pop_ip!(u32);
                        *sp = self.locals[usize::try_from(local_idx).unwrap()];
                        sp = sp.add(8);
                    }
                    OpCode::STORE_LOCAL => {
                        let local_idx: u32 = pop_ip!(u32);
                        sp = sp.offset(-8);
                        self.locals[usize::try_from(local_idx).unwrap()] = *sp;
                    }
                    OpCode::POP64 => {
                        sp = sp.offset(-8);
                    },
                    OpCode::POP128 => {
                        sp = sp.offset(-16);
                    },
                    OpCode::AND => bool_bin_op!(&&),
                    OpCode::OR => bool_bin_op!(||),
                    OpCode::ADD => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => bin_op!(u8 +),
                            Primitive::U16 => bin_op!(u16 +),
                            Primitive::U32 => bin_op!(u32 +),
                            Primitive::U64 => bin_op!(u64 +),
                            Primitive::I8 => bin_op!(i8 +),
                            Primitive::I16 => bin_op!(i16 +),
                            Primitive::I32 => bin_op!(i32 +),
                            Primitive::I64 => bin_op!(i64 +),
                            Primitive::F32 => bin_op!(f32 +),
                            Primitive::F64 => bin_op!(f64 +),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    OpCode::SUB => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => bin_op!(u8 -),
                            Primitive::U16 => bin_op!(u16 -),
                            Primitive::U32 => bin_op!(u32 -),
                            Primitive::U64 => bin_op!(u64 -),
                            Primitive::I8 => bin_op!(i8 -),
                            Primitive::I16 => bin_op!(i16 -),
                            Primitive::I32 => bin_op!(i32 -),
                            Primitive::I64 => bin_op!(i64 -),
                            Primitive::F32 => bin_op!(f32 -),
                            Primitive::F64 => bin_op!(f64 -),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    OpCode::MUL => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => bin_op!(u8 *),
                            Primitive::U16 => bin_op!(u16 *),
                            Primitive::U32 => bin_op!(u32 *),
                            Primitive::U64 => bin_op!(u64 *),
                            Primitive::I8 => bin_op!(i8 *),
                            Primitive::I16 => bin_op!(i16 *),
                            Primitive::I32 => bin_op!(i32 *),
                            Primitive::I64 => bin_op!(i64 *),
                            Primitive::F32 => bin_op!(f32 *),
                            Primitive::F64 => bin_op!(f64 *),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    OpCode::DIV => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => bin_op!(u8 /),
                            Primitive::U16 => bin_op!(u16 /),
                            Primitive::U32 => bin_op!(u32 /),
                            Primitive::U64 => bin_op!(u64 /),
                            Primitive::I8 => bin_op!(i8 /),
                            Primitive::I16 => bin_op!(i16 /),
                            Primitive::I32 => bin_op!(i32 /),
                            Primitive::I64 => bin_op!(i64 /),
                            Primitive::F32 => bin_op!(f32 /),
                            Primitive::F64 => bin_op!(f64 /),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    OpCode::EQ => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::BOOL => bool_bin_op!(==),
                            Primitive::U8 => to_bool_bin_op!(u8 ==),
                            Primitive::U16 => to_bool_bin_op!(u16 ==),
                            Primitive::U32 => to_bool_bin_op!(u32 ==),
                            Primitive::U64 => to_bool_bin_op!(u64 ==),
                            Primitive::I8 => to_bool_bin_op!(i8 ==),
                            Primitive::I16 => to_bool_bin_op!(i16 ==),
                            Primitive::I32 => to_bool_bin_op!(i32 ==),
                            Primitive::I64 => to_bool_bin_op!(i64 ==),
                            Primitive::F32 => to_bool_bin_op!(f32 ==),
                            Primitive::F64 => to_bool_bin_op!(f64 ==),
                        }
                    },
                    OpCode::NEQ => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::BOOL => bool_bin_op!(==),
                            Primitive::U8 => to_bool_bin_op!(u8 !=),
                            Primitive::U16 => to_bool_bin_op!(u16 !=),
                            Primitive::U32 => to_bool_bin_op!(u32 !=),
                            Primitive::U64 => to_bool_bin_op!(u64 !=),
                            Primitive::I8 => to_bool_bin_op!(i8 !=),
                            Primitive::I16 => to_bool_bin_op!(i16 !=),
                            Primitive::I32 => to_bool_bin_op!(i32 !=),
                            Primitive::I64 => to_bool_bin_op!(i64 !=),
                            Primitive::F32 => to_bool_bin_op!(f32 !=),
                            Primitive::F64 => to_bool_bin_op!(f64 !=),
                        }
                    },
                    OpCode::GR => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => to_bool_bin_op!(u8 >),
                            Primitive::U16 => to_bool_bin_op!(u16 >),
                            Primitive::U32 => to_bool_bin_op!(u32 >),
                            Primitive::U64 => to_bool_bin_op!(u64 >),
                            Primitive::I8 => to_bool_bin_op!(i8 >),
                            Primitive::I16 => to_bool_bin_op!(i16 >),
                            Primitive::I32 => to_bool_bin_op!(i32 >),
                            Primitive::I64 => to_bool_bin_op!(i64 >),
                            Primitive::F32 => to_bool_bin_op!(f32 >),
                            Primitive::F64 => to_bool_bin_op!(f64 >),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    OpCode::GR_EQ => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => to_bool_bin_op!(u8 >=),
                            Primitive::U16 => to_bool_bin_op!(u16 >=),
                            Primitive::U32 => to_bool_bin_op!(u32 >=),
                            Primitive::U64 => to_bool_bin_op!(u64 >=),
                            Primitive::I8 => to_bool_bin_op!(i8 >=),
                            Primitive::I16 => to_bool_bin_op!(i16 >=),
                            Primitive::I32 => to_bool_bin_op!(i32 >=),
                            Primitive::I64 => to_bool_bin_op!(i64 >=),
                            Primitive::F32 => to_bool_bin_op!(f32 >=),
                            Primitive::F64 => to_bool_bin_op!(f64 >=),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    OpCode::LE => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => to_bool_bin_op!(u8 <),
                            Primitive::U16 => to_bool_bin_op!(u16 <),
                            Primitive::U32 => to_bool_bin_op!(u32 <),
                            Primitive::U64 => to_bool_bin_op!(u64 <),
                            Primitive::I8 => to_bool_bin_op!(i8 <),
                            Primitive::I16 => to_bool_bin_op!(i16 <),
                            Primitive::I32 => to_bool_bin_op!(i32 <),
                            Primitive::I64 => to_bool_bin_op!(i64 <),
                            Primitive::F32 => to_bool_bin_op!(f32 <),
                            Primitive::F64 => to_bool_bin_op!(f64 <),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    OpCode::LE_EQ => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => to_bool_bin_op!(u8 <=),
                            Primitive::U16 => to_bool_bin_op!(u16 <=),
                            Primitive::U32 => to_bool_bin_op!(u32 <=),
                            Primitive::U64 => to_bool_bin_op!(u64 <=),
                            Primitive::I8 => to_bool_bin_op!(i8 <=),
                            Primitive::I16 => to_bool_bin_op!(i16 <=),
                            Primitive::I32 => to_bool_bin_op!(i32 <=),
                            Primitive::I64 => to_bool_bin_op!(i64 <=),
                            Primitive::F32 => to_bool_bin_op!(f32 <=),
                            Primitive::F64 => to_bool_bin_op!(f64 <=),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    OpCode::TRANSPILE_ADD => {
                        let lsb = pop_sp!().u64;
                        let msb = pop_sp!().u64;
                        let transpiler = pop_sp!();

                        let uuid = Uuid::from_u64_pair(msb, lsb);
                        self.transpile_functions.push(uuid);
                    }
                }
            }
        }
    }
}
