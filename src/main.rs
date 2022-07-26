#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub tenlang);
mod ast;

use std::ffi::OsString;
use std::path::PathBuf;

use clap::{arg, Command};

fn cli() -> Command<'static> {
    Command::new("tenlang")
        .about("A cli implementation for the tenlang language.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("check")
                .about("Parse files to check for validity.")
                .arg_required_else_help(true)
                .arg(arg!(<PATH> ... "files to check").value_parser(clap::value_parser!(PathBuf))),
        )
}

fn main() {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("check", sub_matches)) => {
            let paths = sub_matches
                .get_many::<PathBuf>("PATH")
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();

            for path in paths {
                println!("Checking {:?}...", path);

                let content = std::fs::read_to_string(&path)
                    .expect("could not read file");

                let _ = tenlang::ProgramParser::new()
                    .parse(content.as_str())
                    .unwrap();
            }

            println!("All files are valid .tenlang!");
        },
        _ => unreachable!(),
    }
}
