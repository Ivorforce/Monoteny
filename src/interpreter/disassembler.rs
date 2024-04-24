use std::mem::transmute;
use std::ops::Add;
use std::ptr::read_unaligned;
use crate::interpreter::chunks::Chunk;
use crate::interpreter::opcode::{OpCode, Primitive};

pub fn disassemble(chunk: &Chunk) {
    unsafe {
        let mut idx = 0;

        while idx < chunk.code.len() {
            print!("{:04}\t", idx);
            idx += disassemble_one(transmute(&chunk.code[idx]));
            print!("\n");
        }
    }
}

pub fn disassemble_one(ip: *const u8) -> usize {
    unsafe {
        let code = transmute::<u8, OpCode>(*ip);
        match code {
            OpCode::NEG | OpCode::ADD | OpCode::SUB | OpCode::MUL | OpCode::DIV |
            OpCode::EQ | OpCode::NEQ | OpCode::GR | OpCode::GR_EQ  | OpCode::LE  | OpCode::LE_EQ |
            OpCode::MOD | OpCode::EXP | OpCode::LOG | OpCode::PARSE | OpCode::TO_STRING => {
                print!("{:?}\t{:?}", code, transmute::<u8, Primitive>(*ip.add(1)));
                return 1 + 1;
            },
            OpCode::LOAD8 => {
                print!("{:?}\t{:?}", code, *ip.add(1));
                return 1 + 1;
            }
            OpCode::LOAD16 => {
                print!("{:?}\t{:?}", code, read_unaligned(ip.add(1) as *mut u16));
                return 1 + 2;
            }
            OpCode::LOAD32 | OpCode::LOAD_LOCAL | OpCode::STORE_LOCAL | OpCode::LOAD_CONSTANT => {
                print!("{:?}\t{:?}", code, read_unaligned(ip.add(1) as *mut u32));
                return 1 + 4;
            }
            OpCode::LOAD64 => {
                print!("{:?}\t{:?}", code, read_unaligned(ip.add(1) as *mut u64));
                return 1 + 8;
            }
            OpCode::LOAD128 => {
                print!("{:?}\t{:?}", code, read_unaligned(ip.add(1) as *mut u128));
                return 1 + 16;
            }
            OpCode::NOOP | OpCode::RETURN | OpCode::TRANSPILE_ADD | OpCode::AND | OpCode::OR | OpCode::POP64 | OpCode::POP128 | OpCode::PRINT | OpCode::NOT => {
                print!("{:?}", code);
                return 1;
            },
        }
    }
}
