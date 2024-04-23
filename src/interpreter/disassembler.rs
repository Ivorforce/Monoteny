use std::mem::transmute;
use std::ops::Add;
use std::ptr::read_unaligned;
use crate::interpreter::chunks::{Chunk, Code, Primitive};

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
        let code = transmute::<u8, Code>(*ip);
        match code {
            Code::ADD | Code::SUB | Code::MUL | Code::DIV |
            Code::EQ | Code::NEQ | Code::GR | Code::GR_EQ  | Code::LE  | Code::LE_EQ => {
                print!("{:?}\t{:?}", code, transmute::<u8, Primitive>(*ip.add(1)));
                return 1 + 1;
            },
            Code::LOAD8 => {
                print!("{:?}\t{:?}", code, *ip.add(1));
                return 1 + 1;
            }
            Code::LOAD16 => {
                print!("{:?}\t{:?}", code, read_unaligned(ip.add(1) as *mut u16));
                return 1 + 2;
            }
            Code::LOAD32 | Code::LOAD_LOCAL | Code::STORE_LOCAL => {
                print!("{:?}\t{:?}", code, read_unaligned(ip.add(1) as *mut u32));
                return 1 + 4;
            }
            Code::LOAD64 => {
                print!("{:?}\t{:?}", code, read_unaligned(ip.add(1) as *mut u64));
                return 1 + 8;
            }
            Code::LOAD128 => {
                print!("{:?}\t{:?}", code, read_unaligned(ip.add(1) as *mut u128));
                return 1 + 16;
            }
            Code::NOOP | Code::RETURN | Code::TRANSPILE_ADD | Code::AND | Code::OR | Code::POP32 | Code::POP64 | Code::POP128 => {
                print!("{:?}", code);
                return 1;
            },
        }
    }
}
