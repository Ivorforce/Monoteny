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

use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;
use clap::{arg, Command};
use itertools::Itertools;
use colored::Colorize;
use crate::error::{dump_failure, dump_result, dump_start, dump_success, dump_named_failure, format_errors, RuntimeError};
use crate::interpreter::{Runtime, common};
use crate::program::module::Module;
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
    println!("{}", env::args().join(" "));
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("run", sub_matches)) => {
            let input_path = sub_matches.get_one::<PathBuf>("PATH").unwrap();

            let builtins = program::builtins::create_builtins();
            let mut runtime = Runtime::new(&builtins);
            if let Err(e) = common::load(&mut runtime) {
                _ = dump_start("import(monoteny.common)");
                return dump_failure( e);
            }

            let module = match runtime.load_file(input_path) {
                Ok(m) => m,
                Err(e) => return dump_named_failure(format!("import({})", input_path.as_os_str().to_string_lossy()).as_str(), e),
            };

            dump_result(
                dump_start(format!("{}:@main", input_path.as_os_str().to_string_lossy()).as_str()),
                interpreter::run::main(&module, &mut runtime).map_err(|e| vec![e])
            )
        },
        Some(("check", sub_matches)) => {
            let paths = sub_matches
                .get_many::<PathBuf>("PATH")
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();

            let start = dump_start(format!("check for {} file(s)", paths.len()).as_str());

            let builtins = program::builtins::create_builtins();
            let mut runtime = Runtime::new(&builtins);
            if let Err(e) = common::load(&mut runtime) {
                _ = dump_start("import(monoteny.common)");
                return dump_failure( e);
            }

            let mut error_count = 0;
            for path in paths {
                match runtime.load_file(path) {
                    Ok(_) => {},
                    Err(e) => {
                        dump_named_failure(format!("import({})", path.as_os_str().to_string_lossy()).as_str(), e);
                        error_count += 1;
                    },
                };
            }

            if error_count == 0 {
                dump_success(start);
            }

            ExitCode::from(error_count)
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
            match common::load(&mut runtime) {
                Err(e) => return dump_named_failure("import(monoteny.common)", e),
                _ => {}
            }

            let module = match runtime.load_file(input_path) {
                Ok(m) => m,
                Err(e) => return dump_named_failure(format!("import({})", input_path.as_os_str().to_string_lossy()).as_str(), e),
            };

            let mut error_count = 0;

            for output_extension in output_extensions {
                let start = dump_start(format!("{}:@transpile using {}", input_path.as_os_str().to_string_lossy(), output_extension).as_str());
                match transpile_target(base_filename, base_output_path, should_constant_fold, &mut runtime, &module, output_extension) {
                    Ok(paths) => {
                        for path in paths {
                            println!("{}", path.to_str().unwrap());
                        }
                        dump_success(start);
                    }
                    Err(e) => {
                        dump_failure(e);
                        error_count += 1;
                    },
                }
                println!();
            }

            ExitCode::from(error_count)
        },
        _ => {
            panic!("Unsupported action.")
        },
    }
}

fn transpile_target(base_filename: &str, base_output_path: &Path, should_constant_fold: bool, mut runtime: &mut Box<Runtime>, module: &Box<Module>, output_extension: &str) -> Result<Vec<PathBuf>, Vec<RuntimeError>> {
    let mut context = match output_extension {
        "py" => transpiler::python::create_context(&runtime),
        output_extension => panic!("File type not supported: {}", output_extension)
    };

    let mut transpiler = transpiler::run(&module, &mut runtime, &mut context).map_err(|e| vec![e])?;

    if should_constant_fold {
        transpiler::constant_fold(&mut transpiler);
    }

    let file_map = context.make_files(base_filename, &runtime, &transpiler)?;
    let output_files = file_map.into_iter().map(|(filename, content)| {
        let file_path = base_output_path.join(filename);
        let mut f = File::create(file_path.clone()).expect("Unable to create file");
        let f: &mut (dyn Write) = &mut f;
        write!(f, "{}", content).expect("Error writing file");
        file_path
    }).collect_vec();
    Ok(output_files)
}
