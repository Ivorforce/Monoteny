use std::mem::transmute;
use std::ops::Add;
use std::ptr::read_unaligned;
use crate::interpreter::chunks::{Chunk, Code, Primitive};

pub fn disassemble(chunk: &Chunk) {
    unsafe {
        let mut idx = 0;

        while idx <= chunk.code.len() {
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
            Code::ADD | Code::SUB | Code::MUL | Code::DIV => {
                print!("{:?}\t{:?}", code, transmute::<u8, Primitive>(*ip.add(1)));
                return 2;
            },
            Code::LOAD8 => {
                print!("{:?}\t{:?}", code, *ip.add(1));
                return 2;
            }
            Code::LOAD16 => {
                print!("{:?}\t{:?}", code, read_unaligned(ip.add(1) as *mut u16));
                return 2;
            }
            Code::LOAD32 => {
                print!("{:?}\t{:?}", code, read_unaligned(ip.add(1) as *mut u32));
                return 2;
            }
            Code::LOAD64 => {
                print!("{:?}\t{:?}", code, read_unaligned(ip.add(1) as *mut u64));
                return 2;
            }
            Code::LOAD128 => {
                print!("{:?}\t{:?}", code, read_unaligned(ip.add(1) as *mut u128));
                return 2;
            }
            _ => {
                print!("{:?}", code);
                return 1;
            },
        }
    }
}
