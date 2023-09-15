#[macro_use]
extern crate lalrpop_util;
extern crate core;

lalrpop_mod!(pub monoteny_grammar);
pub mod interpreter;
pub mod linker;
pub mod parser;
pub mod program;
pub mod transpiler;
pub mod util;
pub mod monomorphize;
pub mod integration_tests;
pub mod constant_folding;

use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use clap::{arg, Command};
use crate::interpreter::{Runtime, InterpreterError, common};
use crate::linker::LinkError;


fn cli() -> Command<'static> {
    Command::new("monoteny")
        .about("A cli implementation for the monoteny language.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("run")
                .about("Run a file using the interpreter.")
                .arg_required_else_help(true)
                .arg(arg!(<PATH> "file to run").value_parser(clap::value_parser!(PathBuf)))
        )
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
                .arg(arg!(<OUTPUT> "output file path").required(false).value_parser(clap::value_parser!(PathBuf)).long("output").short('o'))
                .arg(arg!(<ALL> "output using all available transpilers").required(false).takes_value(false).long("all"))
                .arg(arg!(<NOFOLD> "don't use constant folding to shorten the code at compile time").required(false).takes_value(false).long("nofold"))
        )
}

fn main() -> Result<(), InterpreterError> {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("run", sub_matches)) => {
            let path = sub_matches.get_one::<PathBuf>("PATH").unwrap();

            let builtins = program::builtins::create_builtins();
            let mut runtime = Runtime::new(&builtins);
            common::load(&mut runtime)?;

            let module = runtime.load_file(path)?;

            interpreter::run::main(&module, &mut runtime)?;
        },
        Some(("check", sub_matches)) => {
            let paths = sub_matches
                .get_many::<PathBuf>("PATH")
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();

            let builtins = program::builtins::create_builtins();
            let mut runtime = Runtime::new(&builtins);
            common::load(&mut runtime)?;

            for path in paths {
                println!("Checking {:?}...", path);
                runtime.load_file(path)?;
            }

            println!("All files are valid .monoteny!");
        },
        Some(("transpile", sub_matches)) => {
            let input_path = sub_matches.get_one::<PathBuf>("INPUT").unwrap();
            let output_path = match sub_matches.contains_id("OUTPUT") {
                true => sub_matches.get_one::<PathBuf>("OUTPUT").unwrap().clone(),
                false => input_path.with_extension(""),
            };
            let should_output_all = sub_matches.is_present("ALL");
            let should_constant_fold = !sub_matches.is_present("NOFOLD");

            let output_extensions: Vec<&str> = match should_output_all {
                true => vec!["py"],
                false => vec![output_path.extension().and_then(OsStr::to_str).unwrap()]
            };

            let builtins = program::builtins::create_builtins();
            let mut runtime = Runtime::new(&builtins);
            common::load(&mut runtime)?;

            let module = runtime.load_file(input_path)?;

            for output_extension in output_extensions {
                match output_extension {
                    "py" => {
                        let transpiled_tree = transpiler::python::transpile_module(
                            &module, &mut runtime, should_constant_fold
                        )?;

                        let python_path = output_path.with_extension("py");
                        let mut f = File::create(python_path.clone()).expect("Unable to create file");
                        let f: &mut (dyn Write) = &mut f;
                        write!(f, "{}", transpiled_tree).expect("Error writing file");

                        println!("{}", python_path.to_str().unwrap());
                    },
                    _ => unreachable!()
                };
            }
        },
        _ => {
            panic!("Unsupported action.")
        },
    }

    Ok(())
}
