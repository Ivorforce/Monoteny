use std::mem::transmute;
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
        // TODO Somehow, {:<20?} doesn't pad correctly.
        print!("{:<15}", format!("{:?}", code));

        match code {
            OpCode::NEG | OpCode::ADD | OpCode::SUB | OpCode::MUL | OpCode::DIV |
            OpCode::EQ | OpCode::NEQ | OpCode::GR | OpCode::GR_EQ  | OpCode::LE  | OpCode::LE_EQ |
            OpCode::MOD | OpCode::EXP | OpCode::LOG | OpCode::PARSE | OpCode::TO_STRING => {
                print!("\t{:?}", transmute::<u8, Primitive>(*ip.add(1)));
                return 1 + 1;
            },
            OpCode::LOAD8 | OpCode::GET_MEMBER_8 | OpCode::SET_MEMBER_8 | OpCode::ALLOC_8 => {
                print!("\t{:?}", *ip.add(1));
                return 1 + 1;
            }
            OpCode::LOAD16 => {
                print!("\t{:?}", read_unaligned(ip.add(1) as *mut u16));
                return 1 + 2;
            }
            OpCode::LOAD32 | OpCode::LOAD_LOCAL | OpCode::STORE_LOCAL | OpCode::LOAD_CONSTANT => {
                print!("\t{:?}", read_unaligned(ip.add(1) as *mut u32));
                return 1 + 4;
            }
            OpCode::LOAD64 => {
                print!("\t{:?}", read_unaligned(ip.add(1) as *mut u64));
                return 1 + 8;
            }
            OpCode::LOAD128 => {
                print!("\t{:?}", read_unaligned(ip.add(1) as *mut u128));
                return 1 + 16;
            }
            OpCode::JUMP | OpCode::JUMP_IF_FALSE => {
                print!("\t{:?}", read_unaligned(ip.add(1) as *mut i32));
                return 1 + 4;
            }
            OpCode::NOOP | OpCode::PANIC | OpCode::RETURN | OpCode::TRANSPILE_ADD | OpCode::AND |
            OpCode::OR | OpCode::POP64 | OpCode::POP128 | OpCode::PRINT | OpCode::NOT |
            OpCode::ADD_STRING | OpCode::DUP64 => {
                return 1;
            },
        }
    }
}
