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
use std::process::ExitCode;
use clap::{arg, Command};
use log::error;
use crate::interpreter::{Runtime, InterpreterError, common};
use crate::linker::LinkError;
use crate::transpiler::Context;


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

fn main() -> ExitCode {
    match internal_main() {
        Ok(_) => {
            ExitCode::from(0)
        },
        Err(err) => {
            println!("{}", err);
            ExitCode::from(1)
        }
    }
}

fn internal_main() -> Result<(), InterpreterError> {
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
            let output_path_proto = match sub_matches.contains_id("OUTPUT") {
                true => sub_matches.get_one::<PathBuf>("OUTPUT").unwrap().clone(),
                false => input_path.with_extension(""),
            };
            let base_filename = output_path_proto.file_name().and_then(OsStr::to_str).unwrap();
            let base_output_path = output_path_proto.parent().unwrap();

            let should_output_all = sub_matches.is_present("ALL");
            let should_constant_fold = !sub_matches.is_present("NOFOLD");

            let output_extensions: Vec<&str> = match should_output_all {
                true => vec!["py"],
                false => vec![output_path_proto.extension().and_then(OsStr::to_str).unwrap()]
            };

            let builtins = program::builtins::create_builtins();
            let mut runtime = Runtime::new(&builtins);
            common::load(&mut runtime)?;

            let module = runtime.load_file(input_path)?;

            for output_extension in output_extensions {
                let mut context = match output_extension {
                    "py" => transpiler::python::create_context(&runtime),
                    output_extension => panic!("File type not supported: {}", output_extension)
                };

                let mut transpiler = transpiler::run(&module, &mut runtime, &mut context)?;

                if should_constant_fold {
                    transpiler::constant_fold(&mut transpiler);
                }

                let file_map = context.make_files(base_filename, &runtime, &transpiler)?;
                for (filename, content) in file_map {
                    let file_path = base_output_path.join(filename);
                    let mut f = File::create(file_path.clone()).expect("Unable to create file");
                    let f: &mut (dyn Write) = &mut f;
                    write!(f, "{}", content).expect("Error writing file");

                    println!("{}", file_path.to_str().unwrap());
                }
            }
        },
        _ => {
            panic!("Unsupported action.")
        },
    }

    Ok(())
}
