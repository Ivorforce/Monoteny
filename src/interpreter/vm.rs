use std::mem::transmute;
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
            let mut sp: *mut u8 = (&mut self.stack[0] as *mut u8).offset(-1);

            loop {
                disassemble_one(ip);
                print!("\n");

                match transmute::<u8, Code>(*ip) {
                    Code::NOOP => ip = ip.add(1),
                    Code::RETURN => return Ok(()),
                    Code::LOAD8 => {
                        let arg = *ip.add(1);
                        ip = ip.add(2);

                        sp = sp.add(1);
                        *sp = arg;
                    }
                    Code::ADD => {
                        let arg: Primitive = transmute(*ip.add(1));
                        ip = ip.add(2);

                        match arg {
                            Primitive::BOOL => return Err(RuntimeError::new("Cannot add bool.".to_string())),
                            Primitive::U8 => {
                                let lhs = *sp;
                                sp = sp.offset(-1);
                                let rhs = *sp;
                                *sp = lhs + rhs;
                            },
                            _ => todo!(),
                        }
                    },
                    Code::SUB => todo!(),
                    Code::MUL => todo!(),
                    Code::DIV => todo!(),
                }
            }
        }
    }
}
