use std::mem::transmute;
use crate::interpreter::chunks::{Chunk, Code, Primitive};

pub fn disassemble(chunk: &Chunk) {
    let mut ip = 0;
    while ip < chunk.code.len() {
        print!("{:04}\t", ip);
        disassemble_one(&chunk, &mut ip);
        print!("\n");
    }
}

pub fn disassemble_one(chunk: &Chunk, ip: &mut usize) {
    unsafe {
        let code = &chunk.code[*ip];
        match code {
            Code::ADD | Code::SUB | Code::MUL | Code::DIV => {
                print!("{:?}\t{:?}", code, transmute::<u8, Primitive>(chunk.code[*ip + 1] as u8));
                *ip += 2;
            },
            _ => {
                print!("{:?}", code);
                *ip += 1;
            },
        }
    }
}
