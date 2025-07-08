use std::alloc::{alloc, Layout};
use std::cell::RefCell;
use std::mem::transmute;
use std::ops::Neg;
use std::ptr::{read_unaligned, write_unaligned};
use std::rc::{Rc, Weak};

use monoteny_macro::{bin_expr, pop_ip, un_expr};
use uuid::Uuid;

use crate::error::{RResult, RuntimeError};
use crate::interpreter::chunks::Chunk;
use crate::interpreter::compile::compile_server::CompileServer;
use crate::interpreter::data::{string_to_ptr, Value};
use crate::interpreter::opcode::{OpCode, Primitive};
use crate::interpreter::vm::call_frame::CallFrame;
use crate::interpreter::vm::util::pop_stack;

pub mod call_frame;
pub mod util;

pub struct VM {
    pub out: RefCell<Box<dyn std::io::Write>>,
    pub stack: Vec<Value>,
    pub transpile_functions: Vec<Uuid>,
    pub call_frames: Vec<CallFrame>,
}

pub type IntrinsicFunction = fn(&mut VM, &mut *mut Value) -> RResult<()>;

impl VM {
    pub fn new() -> VM {
        VM {
            // TODO This should dynamically resize probably.
            out: RefCell::new(Box::new(std::io::stdout())),
            stack: vec![Value::alloc(); 1024 * 1024],
            transpile_functions: vec![],
            call_frames: Default::default(),
        }
    }

    pub fn run(&mut self, initial_chunk: Rc<Chunk>, compile_server: &CompileServer, parameters: Vec<Value>) -> RResult<Option<Value>> {
        unsafe {
            let mut sp: *mut Value = &mut self.stack[0] as *mut Value;
            for parameter in parameters {
                *sp = parameter;
                sp = sp.add(8);
            }

            let mut fp = sp;

            let mut ip: *const u8 = transmute(&initial_chunk.code[0]);
            let mut current_chunk = initial_chunk;

            loop {
                // println!("sp: {:?}; ip: {:?}", sp, ip);
                // disassemble_one(ip);
                // print!("\n");

                let code = transmute::<u8, OpCode>(*ip);
                ip = ip.add(1);

                match code {
                    OpCode::NOOP => {},
                    OpCode::PANIC => return Err(RuntimeError::error("panic").to_array()),
                    OpCode::RETURN => {
                        if let Some(return_frame) = self.call_frames.pop() {
                            ip = return_frame.ip;
                            fp = return_frame.fp;
                            current_chunk = return_frame.chunk;
                        }
                        else {
                            if sp == &mut self.stack[0] as *mut Value {
                                // No return
                                return Ok(None)
                            }
                            else if sp == (&mut self.stack[0] as *mut Value).offset(8) {
                                // Value return
                                return Ok(Some(*(&mut self.stack[0] as *mut Value)))
                            }
                            else {
                                return Err(RuntimeError::error("Stack was larger than one object at final return.").to_array());
                            }
                        }
                    },
                    OpCode::CALL => {
                        let uuid = Uuid::from_u128(pop_ip!(u128));
                        let chunk = Rc::clone(&compile_server.function_evaluators[&uuid]);
                        self.call_frames.push(CallFrame {
                            chunk: current_chunk,
                            fp,
                            ip
                        });
                        ip = transmute(&chunk.code[0]);
                        fp = sp;
                        current_chunk = chunk;
                    }
                    OpCode::CALL_INTRINSIC => {
                        // TODO Should be platform dependent int (32bit / 64bit)
                        let fun_ptr_int = pop_ip!(u64);
                        let fun: IntrinsicFunction = transmute(fun_ptr_int);
                        fun(self, &mut sp)?;
                    }
                    OpCode::LOAD0 => {
                        sp = sp.add(8);
                    }
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
                    OpCode::LOAD_LOCAL_32 => {
                        let local_idx: i32 = pop_ip!(i32);
                        *sp = *fp.offset(isize::try_from(local_idx).unwrap() * 8);
                        sp = sp.add(8);
                    }
                    OpCode::STORE_LOCAL_32 => {
                        let local_idx: i32 = pop_ip!(i32);
                        sp = sp.offset(-8);
                        *fp.offset(isize::try_from(local_idx).unwrap() * 8) = *sp;
                    }
                    OpCode::LOAD_CONSTANT_32 => {
                        let constant_idx: u32 = pop_ip!(u32);
                        *sp = current_chunk.constants[usize::try_from(constant_idx).unwrap()];
                        sp = sp.add(8);
                    }
                    OpCode::DUP64 => {
                        *sp = *sp.offset(-8);
                        sp = sp.offset(8);
                    }
                    OpCode::POP64 => {
                        sp = sp.offset(-8);
                    },
                    OpCode::SWAP64 => {
                        std::ptr::swap(sp.offset(-16), sp.offset(-8));
                    }
                    OpCode::JUMP => {
                        let jump_distance: i32 = pop_ip!(i32);
                        ip = ip.offset(isize::try_from(jump_distance).unwrap());
                    }
                    OpCode::JUMP_IF_FALSE => {
                        let jump_distance: i32 = pop_ip!(i32);
                        let condition = pop_stack(&mut sp).bool;
                        if !condition {
                            ip = ip.offset(isize::try_from(jump_distance).unwrap());
                        }
                    }
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
                            _ => return Err(RuntimeError::error("Unexpected primitive.").to_array()),
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
                            _ => return Err(RuntimeError::error("Unexpected primitive.").to_array()),
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
                            _ => return Err(RuntimeError::error("Unexpected primitive.").to_array()),
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
                            _ => return Err(RuntimeError::error("Unexpected primitive.").to_array()),
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
                            _ => return Err(RuntimeError::error("Unexpected primitive.").to_array()),
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
                            _ => return Err(RuntimeError::error("Unexpected primitive.").to_array()),
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
                            _ => return Err(RuntimeError::error("Unexpected primitive.").to_array()),
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
                            _ => return Err(RuntimeError::error("Unexpected primitive.").to_array()),
                        }
                    },
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
                            _ => return Err(RuntimeError::error("Unexpected primitive.").to_array()),
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
                            _ => return Err(RuntimeError::error("Unexpected primitive.").to_array()),
                        }
                    }
                    OpCode::EXP => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::U8 => bin_expr!(u8, u8, lhs.wrapping_pow(rhs.into())),
                            Primitive::U16 => bin_expr!(u16, u16, lhs.wrapping_pow(rhs.into())),
                            Primitive::U32 => bin_expr!(u32, u32, lhs.wrapping_pow(rhs.try_into().unwrap())),
                            Primitive::U64 => bin_expr!(u64, u64, lhs.wrapping_pow(rhs.try_into().unwrap())),
                            Primitive::I8 => bin_expr!(i8, i8, lhs.wrapping_pow(rhs.try_into().unwrap())),
                            Primitive::I16 => bin_expr!(i16, i16, lhs.wrapping_pow(rhs.try_into().unwrap())),
                            Primitive::I32 => bin_expr!(i32, i32, lhs.wrapping_pow(rhs.try_into().unwrap())),
                            Primitive::I64 => bin_expr!(i64, i64, lhs.wrapping_pow(rhs.try_into().unwrap())),
                            Primitive::F32 => bin_expr!(f32, f32, lhs.powf(rhs)),
                            Primitive::F64 => bin_expr!(f64, f64, lhs.powf(rhs)),
                            _ => return Err(RuntimeError::error("Unexpected primitive.").to_array()),
                        }
                    }
                    OpCode::LOG => {
                        let arg: Primitive = transmute(pop_ip!(u8));

                        match arg {
                            Primitive::F32 => bin_expr!(f32, f32, lhs.log(rhs)),
                            Primitive::F64 => bin_expr!(f64, f64, lhs.log(rhs)),
                            _ => return Err(RuntimeError::error("Unexpected primitive.").to_array()),
                        }
                    }
                    OpCode::ALLOC_32 => {
                        let size = pop_ip!(u32);
                        let layout = Layout::from_size_align(usize::try_from(size).unwrap() * 8, 8).unwrap();

                        (*sp).ptr = alloc(layout) as *mut ();
                        sp = sp.offset(8);
                    }
                    OpCode::GET_MEMBER_32 => {
                        let slot_idx = pop_ip!(u32);
                        let sp_last = sp.offset(-8);
                        let slot_ptr = (*sp_last).ptr.byte_add(usize::try_from(slot_idx).unwrap() * 8);

                        *sp_last = read_unaligned(slot_ptr as *mut Value);
                    }
                    OpCode::SET_MEMBER_32 => {
                        let slot_idx = pop_ip!(u32);
                        let value = pop_stack(&mut sp);
                        let obj_ptr = pop_stack(&mut sp).ptr;
                        let slot_ptr = obj_ptr.byte_add(usize::try_from(slot_idx).unwrap() * 8);

                        write_unaligned(slot_ptr as *mut Value, value);
                    }
                }
            }
        }
    }
}
