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
pub mod error;

use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;
use clap::{arg, Command};
use itertools::Itertools;
use colored::Colorize;
use crate::error::RuntimeError;
use crate::interpreter::{Runtime, common};
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
                .arg(arg!(<ALL> "use all available transpilers").required(false).takes_value(false).long("all"))
                .arg(arg!(<NOFOLD> "don't use constant folding to shorten the code at compile time").required(false).takes_value(false).long("nofold"))
        )
}

fn main() -> ExitCode {
    match internal_main() {
        Ok(_) => {
            ExitCode::from(0)
        },
        Err(err) => {
            match &err[..] {
                [] => unreachable!(),
                [err] => {
                    println!("Failed to run command:\n\n{}", err);
                }
                errs => {
                    println!("Failed to run command ({} errors):\n\n{}", err.len(), err.into_iter().map(|e| e.to_string()).join("\n\n"));
                }
            }
            ExitCode::from(1)
        }
    }
}

fn internal_main() -> Result<(), Vec<RuntimeError>> {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("run", sub_matches)) => {
            let path = sub_matches.get_one::<PathBuf>("PATH").unwrap();

            let builtins = program::builtins::create_builtins();
            let mut runtime = Runtime::new(&builtins);
            common::load(&mut runtime)?;

            let module = runtime.load_file(path)?;

            println!("{} {}:@main", "Running".green().bold(), path.as_os_str().to_string_lossy());
            let start = Instant::now();
            interpreter::run::main(&module, &mut runtime).map_err(|e| vec![e])?;
            println!("{} running in {:.2}s", "Finished".green().bold(), start.elapsed().as_secs_f32());
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
                println!("{} {}:@check", "Running".green().bold(), path.as_os_str().to_string_lossy());
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

            let target_count = output_extensions.len();
            let start = Instant::now();
            for output_extension in output_extensions {
                let mut context = match output_extension {
                    "py" => transpiler::python::create_context(&runtime),
                    output_extension => panic!("File type not supported: {}", output_extension)
                };

                println!("{} transpile({:?}) using {}:@transpile", "Running".green().bold(), output_extension, input_path.as_os_str().to_string_lossy());
                let mut transpiler = transpiler::run(&module, &mut runtime, &mut context).map_err(|e| vec![e])?;

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
                println!();
            }
            println!("{} transpiling {} target{} in {:.2}s", "Finished".green().bold(), target_count, if target_count == 1 { "" } else { "s" }, start.elapsed().as_secs_f32());
        },
        _ => {
            panic!("Unsupported action.")
        },
    }

    Ok(())
}
