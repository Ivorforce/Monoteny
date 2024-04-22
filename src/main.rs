extern crate core;
#[macro_use]
extern crate lalrpop_util;

use std::env;
use std::process::ExitCode;
use std::ptr::read_unaligned;

use clap::{arg, Command};
use itertools::Itertools;

use crate::error::{dump_failure, RResult};
use crate::interpreter::chunks::{Chunk, Code, Primitive};
use crate::interpreter::disassembler::disassemble;
use crate::interpreter::vm::VM;

lalrpop_mod!(pub monoteny_grammar);
pub mod interpreter;
pub mod resolver;
pub mod parser;
pub mod program;
pub mod transpiler;
pub mod util;
pub mod error;
pub mod repository;
pub mod refactor;
pub mod source;
pub mod cli;

fn cli() -> Command<'static> {
    Command::new("monoteny")
        .about("A cli implementation for the monoteny language.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(cli::run::make_command())
        .subcommand(cli::check::make_command())
        .subcommand(cli::transpile::make_command())
        .subcommand(make_vm_test_command())
}

fn make_vm_test_command() -> Command<'static> {
    Command::new("vm-test")
        .about("Test the vm.")
}

fn run_vm_test() -> RResult<ExitCode> {
    let mut chunk = Chunk::new();
    chunk.push_with_u16(Code::LOAD16, 2);
    chunk.push_with_u16(Code::LOAD16, 6);
    chunk.push_with_u8(Code::ADD, Primitive::U32 as u8);
    chunk.push_with_u16(Code::LOAD16, 8);
    chunk.push_with_u8(Code::EQ, Primitive::U32 as u8);
    chunk.push(Code::RETURN);
    let mut vm = VM::new(&chunk);
    vm.run()?;
    unsafe {
        println!("R: {}", read_unaligned(vm.stack.as_ptr() as *const u16));
    }
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    println!("{}", env::args().join(" "));
    let matches = cli().get_matches();

    let result = match matches.subcommand() {
        Some(("run", sub_matches)) => cli::run::run(sub_matches),
        Some(("check", sub_matches)) => cli::check::run(sub_matches),
        Some(("transpile", sub_matches)) => cli::transpile::run(sub_matches),
        Some(("vm-test", sub_matches)) => run_vm_test(),
        _ => panic!("Unsupported action."),
    };

    match result {
        Ok(c) => c,
        Err(e) => dump_failure(e),
    }
}
