extern crate core;
#[macro_use]
extern crate lalrpop_util;

use std::env;
use std::process::ExitCode;

use clap::Command;
use itertools::Itertools;

use crate::error::dump_failure;

lalrpop_mod!(pub monoteny_grammar);
pub mod interpreter;
pub mod linker;
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
}

fn main() -> ExitCode {
    println!("{}", env::args().join(" "));
    let matches = cli().get_matches();

    let result = match matches.subcommand() {
        Some(("run", sub_matches)) => cli::run::run(sub_matches),
        Some(("check", sub_matches)) => cli::check::run(sub_matches),
        Some(("transpile", sub_matches)) => cli::transpile::run(sub_matches),
        _ => panic!("Unsupported action."),
    };

    match result {
        Ok(c) => c,
        Err(e) => dump_failure(e),
    }
}
