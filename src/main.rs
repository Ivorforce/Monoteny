extern crate core;
#[macro_use]
extern crate lalrpop_util;

use std::env;
use std::process::ExitCode;

use itertools::Itertools;

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
pub mod static_analysis;
pub mod ast;

fn main() -> ExitCode {
    println!("{}", env::args().join(" "));
    cli::run_command()
}
