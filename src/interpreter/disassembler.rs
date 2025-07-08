use std::mem::transmute;
use uuid::Uuid;
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
        print!("{:<20}", format!("{:?}", code));

        match code {
            OpCode::NEG | OpCode::ADD | OpCode::SUB | OpCode::MUL | OpCode::DIV |
            OpCode::EQ | OpCode::NEQ | OpCode::GR | OpCode::GR_EQ  | OpCode::LE  | OpCode::LE_EQ |
            OpCode::MOD | OpCode::EXP | OpCode::LOG => {
                print!("\t{:?}", transmute::<u8, Primitive>(*ip.add(1)));
                return 1 + 1;
            },
            OpCode::LOAD8 => {
                print!("\t{:?}", *ip.add(1));
                return 1 + 1;
            }
            OpCode::LOAD16 => {
                print!("\t{:?}", *(ip.add(1) as *mut u16));
                return 1 + 2;
            }
            OpCode::LOAD32 | OpCode::LOAD_CONSTANT_32 | OpCode::GET_MEMBER_32 | OpCode::SET_MEMBER_32 | OpCode::ALLOC_32 => {
                print!("\t{:?}", *(ip.add(1) as *mut u32));
                return 1 + 4;
            }
            OpCode::LOAD64 | OpCode::CALL_INTRINSIC => {
                print!("\t{:?}", *(ip.add(1) as *mut u64));
                return 1 + 8;
            }
            OpCode::JUMP | OpCode::JUMP_IF_FALSE | OpCode::LOAD_LOCAL_32 | OpCode::STORE_LOCAL_32 => {
                print!("\t{:?}", *(ip.add(1) as *mut i32));
                return 1 + 4;
            }
            OpCode::CALL => {
                print!("\t{}", Uuid::from_u128(*(ip.add(1) as *mut u128)));
                return 1 + 16;
            }
            OpCode::NOOP | OpCode::PANIC | OpCode::RETURN | OpCode::AND |
            OpCode::OR | OpCode::POP64 | OpCode::NOT |
            OpCode::DUP64 | OpCode::LOAD0 | OpCode::SWAP64 => {
                return 1;
            },
        }
    }
}
