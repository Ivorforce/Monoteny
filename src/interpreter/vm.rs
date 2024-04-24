use std::alloc::{alloc, Layout};
use std::mem::transmute;
use monoteny_macro::{bin_expr, pop_ip, pop_sp, un_expr};
use std::ptr::{read_unaligned, write_unaligned};
use itertools::Itertools;
use uuid::Uuid;
use std::ops::Neg;
use crate::error::{RResult, RuntimeError};
use crate::interpreter::chunks::Chunk;
use crate::interpreter::data::{string_to_ptr, Value};
use crate::interpreter::opcode::{OpCode, Primitive};

pub struct VM<'a> {
    pub chunk: &'a Chunk,
    pub stack: Vec<Value>,
    pub locals: Vec<Value>,
    pub transpile_functions: Vec<Uuid>,
}

pub unsafe fn to_str_ptr<A: ToString>(a: A) -> *mut () {
    let string = a.to_string();
    string_to_ptr(&string)
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
                    OpCode::LOAD_CONSTANT => {
                        let constant_idx: u32 = pop_ip!(u32);
                        *sp = self.chunk.constants[usize::try_from(constant_idx).unwrap()];
                        sp = sp.add(8);
                    }
                    OpCode::POP64 => {
                        sp = sp.offset(-8);
                    },
                    OpCode::POP128 => {
                        sp = sp.offset(-16);
                    },
                    OpCode::AND => bin_expr!(bool, bool, lhs&&rhs),
                    OpCode::OR => bin_expr!(bool, bool, lhs||rhs),
                    OpCode::NOT => un_expr!(bool, bool, !val),
                    OpCode::ADD => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => bin_expr!(u8, u8, lhs.wrapping_add(rhs)),
                            Primitive::U16 => bin_expr!(u16, u16, lhs.wrapping_add(rhs)),
                            Primitive::U32 => bin_expr!(u32, u32, lhs.wrapping_add(rhs)),
                            Primitive::U64 => bin_expr!(u64, u64, lhs.wrapping_add(rhs)),
                            Primitive::I8 => bin_expr!(i8, i8, lhs.wrapping_add(rhs)),
                            Primitive::I16 => bin_expr!(i16, i16, lhs.wrapping_add(rhs)),
                            Primitive::I32 => bin_expr!(i32, i32, lhs.wrapping_add(rhs)),
                            Primitive::I64 => bin_expr!(i64, i64, lhs.wrapping_add(rhs)),
                            Primitive::F32 => bin_expr!(f32, f32, lhs+rhs),
                            Primitive::F64 => bin_expr!(f64, f64, lhs+rhs),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    OpCode::SUB => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => bin_expr!(u8, u8, lhs.wrapping_sub(rhs)),
                            Primitive::U16 => bin_expr!(u16, u16, lhs.wrapping_sub(rhs)),
                            Primitive::U32 => bin_expr!(u32, u32, lhs.wrapping_sub(rhs)),
                            Primitive::U64 => bin_expr!(u64, u64, lhs.wrapping_sub(rhs)),
                            Primitive::I8 => bin_expr!(i8, i8, lhs.wrapping_sub(rhs)),
                            Primitive::I16 => bin_expr!(i16, i16, lhs.wrapping_sub(rhs)),
                            Primitive::I32 => bin_expr!(i32, i32, lhs.wrapping_sub(rhs)),
                            Primitive::I64 => bin_expr!(i64, i64, lhs.wrapping_sub(rhs)),
                            Primitive::F32 => bin_expr!(f32, f32, lhs-rhs),
                            Primitive::F64 => bin_expr!(f64, f64, lhs-rhs),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    OpCode::MUL => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => bin_expr!(u8, u8, lhs.wrapping_mul(rhs)),
                            Primitive::U16 => bin_expr!(u16, u16, lhs.wrapping_mul(rhs)),
                            Primitive::U32 => bin_expr!(u32, u32, lhs.wrapping_mul(rhs)),
                            Primitive::U64 => bin_expr!(u64, u64, lhs.wrapping_mul(rhs)),
                            Primitive::I8 => bin_expr!(i8, i8, lhs.wrapping_mul(rhs)),
                            Primitive::I16 => bin_expr!(i16, i16, lhs.wrapping_mul(rhs)),
                            Primitive::I32 => bin_expr!(i32, i32, lhs.wrapping_mul(rhs)),
                            Primitive::I64 => bin_expr!(i64, i64, lhs.wrapping_mul(rhs)),
                            Primitive::F32 => bin_expr!(f32, f32, lhs*rhs),
                            Primitive::F64 => bin_expr!(f64, f64, lhs*rhs),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    OpCode::DIV => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => bin_expr!(u8, u8, lhs/rhs),
                            Primitive::U16 => bin_expr!(u16, u16, lhs/rhs),
                            Primitive::U32 => bin_expr!(u32, u32, lhs/rhs),
                            Primitive::U64 => bin_expr!(u64, u64, lhs/rhs),
                            Primitive::I8 => bin_expr!(i8, i8, lhs/rhs),
                            Primitive::I16 => bin_expr!(i16, i16, lhs/rhs),
                            Primitive::I32 => bin_expr!(i32, i32, lhs/rhs),
                            Primitive::I64 => bin_expr!(i64, i64, lhs/rhs),
                            Primitive::F32 => bin_expr!(f32, f32, lhs/rhs),
                            Primitive::F64 => bin_expr!(f64, f64, lhs/rhs),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    OpCode::EQ => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::BOOL => bin_expr!(bool, bool, lhs==rhs),
                            Primitive::U8 => bin_expr!(u8, bool, lhs==rhs),
                            Primitive::U16 => bin_expr!(u16, bool, lhs==rhs),
                            Primitive::U32 => bin_expr!(u32, bool, lhs==rhs),
                            Primitive::U64 => bin_expr!(u64, bool, lhs==rhs),
                            Primitive::I8 => bin_expr!(i8, bool, lhs==rhs),
                            Primitive::I16 => bin_expr!(i16, bool, lhs==rhs),
                            Primitive::I32 => bin_expr!(i32, bool, lhs==rhs),
                            Primitive::I64 => bin_expr!(i64, bool, lhs==rhs),
                            Primitive::F32 => bin_expr!(f32, bool, lhs==rhs),
                            Primitive::F64 => bin_expr!(f64, bool, lhs==rhs),
                        }
                    },
                    OpCode::NEQ => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::BOOL => bin_expr!(bool, bool, lhs!=rhs),
                            Primitive::U8 => bin_expr!(u8, bool, lhs!=rhs),
                            Primitive::U16 => bin_expr!(u16, bool, lhs!=rhs),
                            Primitive::U32 => bin_expr!(u32, bool, lhs!=rhs),
                            Primitive::U64 => bin_expr!(u64, bool, lhs!=rhs),
                            Primitive::I8 => bin_expr!(i8, bool, lhs!=rhs),
                            Primitive::I16 => bin_expr!(i16, bool, lhs!=rhs),
                            Primitive::I32 => bin_expr!(i32, bool, lhs!=rhs),
                            Primitive::I64 => bin_expr!(i64, bool, lhs!=rhs),
                            Primitive::F32 => bin_expr!(f32, bool, lhs!=rhs),
                            Primitive::F64 => bin_expr!(f64, bool, lhs!=rhs),
                        }
                    },
                    OpCode::GR => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => bin_expr!(u8, bool, lhs>rhs),
                            Primitive::U16 => bin_expr!(u16, bool, lhs>rhs),
                            Primitive::U32 => bin_expr!(u32, bool, lhs>rhs),
                            Primitive::U64 => bin_expr!(u64, bool, lhs>rhs),
                            Primitive::I8 => bin_expr!(i8, bool, lhs>rhs),
                            Primitive::I16 => bin_expr!(i16, bool, lhs>rhs),
                            Primitive::I32 => bin_expr!(i32, bool, lhs>rhs),
                            Primitive::I64 => bin_expr!(i64, bool, lhs>rhs),
                            Primitive::F32 => bin_expr!(f32, bool, lhs>rhs),
                            Primitive::F64 => bin_expr!(f64, bool, lhs>rhs),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    OpCode::GR_EQ => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => bin_expr!(u8, bool, lhs>=rhs),
                            Primitive::U16 => bin_expr!(u16, bool, lhs>=rhs),
                            Primitive::U32 => bin_expr!(u32, bool, lhs>=rhs),
                            Primitive::U64 => bin_expr!(u64, bool, lhs>=rhs),
                            Primitive::I8 => bin_expr!(i8, bool, lhs>=rhs),
                            Primitive::I16 => bin_expr!(i16, bool, lhs>=rhs),
                            Primitive::I32 => bin_expr!(i32, bool, lhs>=rhs),
                            Primitive::I64 => bin_expr!(i64, bool, lhs>=rhs),
                            Primitive::F32 => bin_expr!(f32, bool, lhs>=rhs),
                            Primitive::F64 => bin_expr!(f64, bool, lhs>=rhs),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    OpCode::LE => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => bin_expr!(u8, bool, lhs<rhs),
                            Primitive::U16 => bin_expr!(u16, bool, lhs<rhs),
                            Primitive::U32 => bin_expr!(u32, bool, lhs<rhs),
                            Primitive::U64 => bin_expr!(u64, bool, lhs<rhs),
                            Primitive::I8 => bin_expr!(i8, bool, lhs<rhs),
                            Primitive::I16 => bin_expr!(i16, bool, lhs<rhs),
                            Primitive::I32 => bin_expr!(i32, bool, lhs<rhs),
                            Primitive::I64 => bin_expr!(i64, bool, lhs<rhs),
                            Primitive::F32 => bin_expr!(f32, bool, lhs<rhs),
                            Primitive::F64 => bin_expr!(f64, bool, lhs<rhs),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    },
                    OpCode::LE_EQ => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => bin_expr!(u8, bool, lhs<=rhs),
                            Primitive::U16 => bin_expr!(u16, bool, lhs<=rhs),
                            Primitive::U32 => bin_expr!(u32, bool, lhs<=rhs),
                            Primitive::U64 => bin_expr!(u64, bool, lhs<=rhs),
                            Primitive::I8 => bin_expr!(i8, bool, lhs<=rhs),
                            Primitive::I16 => bin_expr!(i16, bool, lhs<=rhs),
                            Primitive::I32 => bin_expr!(i32, bool, lhs<=rhs),
                            Primitive::I64 => bin_expr!(i64, bool, lhs<=rhs),
                            Primitive::F32 => bin_expr!(f32, bool, lhs<=rhs),
                            Primitive::F64 => bin_expr!(f64, bool, lhs<=rhs),
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
                    OpCode::PRINT => {
                        let ptr = pop_sp!().ptr;
                        let string: *mut String = (ptr as *mut String).clone();
                        println!("{}", *string);
                    }
                    OpCode::NEG => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => un_expr!(u8, u8, val.wrapping_neg()),
                            Primitive::U16 => un_expr!(u16, u16, val.wrapping_neg()),
                            Primitive::U32 => un_expr!(u32, u32, val.wrapping_neg()),
                            Primitive::U64 => un_expr!(u64, u64, val.wrapping_neg()),
                            Primitive::I8 => un_expr!(i8, i8, val.wrapping_neg()),
                            Primitive::I16 => un_expr!(i16, i16, val.wrapping_neg()),
                            Primitive::I32 => un_expr!(i32, i32, val.wrapping_neg()),
                            Primitive::I64 => un_expr!(i64, i64, val.wrapping_neg()),
                            Primitive::F32 => un_expr!(f32, f32, val.neg()),
                            Primitive::F64 => un_expr!(f64, f64, val.neg()),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    }
                    OpCode::MOD => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => bin_expr!(u8, u8, lhs%rhs),
                            Primitive::U16 => bin_expr!(u16, u16, lhs%rhs),
                            Primitive::U32 => bin_expr!(u32, u32, lhs%rhs),
                            Primitive::U64 => bin_expr!(u64, u64, lhs%rhs),
                            Primitive::I8 => bin_expr!(i8, i8, lhs%rhs),
                            Primitive::I16 => bin_expr!(i16, i16, lhs%rhs),
                            Primitive::I32 => bin_expr!(i32, i32, lhs%rhs),
                            Primitive::I64 => bin_expr!(i64, i64, lhs%rhs),
                            Primitive::F32 => bin_expr!(f32, f32, lhs%rhs),
                            Primitive::F64 => bin_expr!(f64, f64, lhs%rhs),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    }
                    OpCode::EXP => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => bin_expr!(u8, u8, lhs.wrapping_pow(rhs.into())),
                            Primitive::U16 => bin_expr!(u16, u16, lhs.wrapping_pow(rhs.into())),
                            Primitive::U32 => bin_expr!(u32, u32, lhs.wrapping_pow(rhs.try_into().unwrap())),
                            Primitive::U64 => bin_expr!(u64, u64, lhs.wrapping_pow(rhs.try_into().unwrap())),
                            Primitive::I8 => bin_expr!(i8, i8, lhs.wrapping_pow(rhs.into())),
                            Primitive::I16 => bin_expr!(i16, i16, lhs.wrapping_pow(rhs.into())),
                            Primitive::I32 => bin_expr!(i32, i32, lhs.wrapping_pow(rhs.try_into().unwrap())),
                            Primitive::I64 => bin_expr!(i64, i64, lhs.wrapping_pow(rhs.try_into().unwrap())),
                            Primitive::F32 => bin_expr!(f32, f32, lhs.powf(rhs)),
                            Primitive::F64 => bin_expr!(f64, f64, lhs.powf(rhs)),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    }
                    OpCode::LOG => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::F32 => bin_expr!(f32, f32, lhs.log(rhs)),
                            Primitive::F64 => bin_expr!(f64, f64, lhs.log(rhs)),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    }
                    OpCode::PARSE => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        let sp_last = sp.offset(-8);
                        let string = (*((*sp_last).ptr as *mut String)).as_str();
                        let i: i32 = string.parse().unwrap();

                        match arg {
                            Primitive::U8 => (*sp_last).u8 = string.parse().unwrap(),
                            Primitive::U16 => (*sp_last).u16 = string.parse().unwrap(),
                            Primitive::U32 => (*sp_last).u32 = string.parse().unwrap(),
                            Primitive::U64 => (*sp_last).u64 = string.parse().unwrap(),
                            Primitive::I8 => (*sp_last).i8 = string.parse().unwrap(),
                            Primitive::I16 => (*sp_last).i16 = string.parse().unwrap(),
                            Primitive::I32 => (*sp_last).i32 = string.parse().unwrap(),
                            Primitive::I64 => (*sp_last).i64 = string.parse().unwrap(),
                            Primitive::F32 => (*sp_last).f32 = string.parse().unwrap(),
                            Primitive::F64 => (*sp_last).f64 = string.parse().unwrap(),
                            _ => return Err(RuntimeError::new("Unexpected primitive.".to_string())),
                        }
                    }
                    OpCode::TO_STRING => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => un_expr!(u8, ptr, to_str_ptr(val)),
                            Primitive::U16 => un_expr!(u16, ptr, to_str_ptr(val)),
                            Primitive::U32 => un_expr!(u32, ptr, to_str_ptr(val)),
                            Primitive::U64 => un_expr!(u64, ptr, to_str_ptr(val)),
                            Primitive::I8 => un_expr!(i8, ptr, to_str_ptr(val)),
                            Primitive::I16 => un_expr!(i16, ptr, to_str_ptr(val)),
                            Primitive::I32 => un_expr!(i32, ptr, to_str_ptr(val)),
                            Primitive::I64 => un_expr!(i64, ptr, to_str_ptr(val)),
                            Primitive::F32 => un_expr!(f32, ptr, to_str_ptr(val)),
                            Primitive::F64 => un_expr!(f64, ptr, to_str_ptr(val)),
                            Primitive::BOOL => un_expr!(bool, ptr, to_str_ptr(val)),
                        }
                    }
                }
            }
        }
    }
}
