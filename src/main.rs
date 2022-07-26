#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub tenlang);
mod abstract_syntax;
mod computation_tree;
mod languages;

use std::ffi::OsStr;
use std::fs::File;
use std::path::PathBuf;
use std::process::exit;

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
                .arg(arg!(<PATH> ... "files to check").value_parser(clap::value_parser!(PathBuf)))
        )
        .subcommand(
            Command::new("transpile")
                .about("Transpile a file into another language.")
                .arg_required_else_help(true)
                .arg(arg!(<INPUT> "file to transpile").value_parser(clap::value_parser!(PathBuf)).long("input").short('i'))
                .arg(arg!(<OUTPUT> "output file path").value_parser(clap::value_parser!(PathBuf)).long("output").short('o'))
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
        Some(("transpile", sub_matches)) => {
            let input_path = sub_matches.get_one::<PathBuf>("INPUT").unwrap();
            let output_path = sub_matches.get_one::<PathBuf>("OUTPUT").unwrap();

            let transpiler: Box<dyn languages::transpiler::Transpiler> = match output_path.extension().and_then(OsStr::to_str) {
                Some("py") => Box::new(languages::python::PythonTranspiler {}),
                _ => {
                    println!("Output path must have a known extension.");
                    exit(1)
                }
            };

            let content = std::fs::read_to_string(&input_path)
                .expect("could not read file");

            let abstract_syntax_tree = tenlang::ProgramParser::new()
                .parse(content.as_str())
                .unwrap();

            let computation_tree = computation_tree::analyze_program(abstract_syntax_tree);

            let mut f = File::create(output_path).expect("Unable to create file");
            transpiler.transpile(computation_tree, &mut f).expect("Error when writing to file");

            println!("Transpiled file to {:?}!", output_path);
        },
        _ => unreachable!(),
    }
}
