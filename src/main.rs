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
pub mod integration_tests;
pub mod error;
pub mod repository;
pub mod graphs;
pub mod refactor;

use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use clap::{arg, Command};
use itertools::Itertools;
use colored::Colorize;
use crate::error::{dump_failure, dump_named_failure, dump_result, dump_start, dump_success, RResult};
use crate::interpreter::Runtime;
use crate::program::module::{Module, module_name};
use crate::transpiler::LanguageContext;


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
                .arg(arg!(<NOREFACTOR> "don't use ANY refactoring").required(false).takes_value(false).long("norefactor"))
                .arg(arg!(<NOFOLD> "don't use constant folding").required(false).takes_value(false).long("nofold"))
                .arg(arg!(<NOINLINE> "don't use inlining").required(false).takes_value(false).long("noinline"))
                .arg(arg!(<NOTRIMLOCALS> "don't trim unused locals code").required(false).takes_value(false).long("notrimlocals"))
        )
}

fn main() -> ExitCode {
    println!("{}", env::args().join(" "));
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("run", sub_matches)) => {
            let input_path = sub_matches.get_one::<PathBuf>("PATH").unwrap();

            let mut runtime = match Runtime::new() {
                Ok(r) => r,
                Err(e) => {
                    _ = dump_start("import(monoteny.core)");
                    return dump_failure( e);
                }
            };
            runtime.repository.add("common", PathBuf::from("monoteny"));

            let module = match runtime.load_file(input_path, module_name("main")) {
                Ok(m) => m,
                Err(e) => return dump_named_failure(format!("import({})", input_path.as_os_str().to_string_lossy()).as_str(), e),
            };

            dump_result(
                dump_start(format!("{}:@main", input_path.as_os_str().to_string_lossy()).as_str()),
                interpreter::run::main(&module, &mut runtime)
            )
        },
        Some(("check", sub_matches)) => {
            let paths = sub_matches
                .get_many::<PathBuf>("PATH")
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();

            let start = dump_start(format!("check for {} file(s)", paths.len()).as_str());

            let mut runtime = match Runtime::new() {
                Ok(r) => r,
                Err(e) => {
                    _ = dump_start("import(monoteny.core)");
                    return dump_failure( e);
                }
            };
            runtime.repository.add("common", PathBuf::from("monoteny"));

            let mut error_count = 0;
            for path in paths {
                match runtime.load_file(path, module_name("main")) {
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

            let can_refactor = !sub_matches.is_present("NOREFACTOR");
            let config = transpiler::Config {
                should_constant_fold: can_refactor && !sub_matches.is_present("NOFOLD"),
                should_monomorphize: true, // TODO Cannot do without it for now
                should_inline: can_refactor && !sub_matches.is_present("NOINLINE"),
                should_trim_locals: can_refactor && !sub_matches.is_present("NOTRIMLOCALS"),
            };
            let should_output_all = sub_matches.is_present("ALL");

            let output_extensions: Vec<&str> = match should_output_all {
                true => vec!["py"],
                false => vec![output_path_proto.extension().and_then(OsStr::to_str).unwrap()]
            };

            let mut runtime = match Runtime::new() {
                Ok(r) => r,
                Err(e) => {
                    _ = dump_start("import(monoteny.core)");
                    return dump_failure( e);
                }
            };
            runtime.repository.add("common", PathBuf::from("monoteny"));

            let module = match runtime.load_file(input_path, module_name("main")) {
                Ok(m) => m,
                Err(e) => return dump_named_failure(format!("import({})", input_path.as_os_str().to_string_lossy()).as_str(), e),
            };

            let mut error_count = 0;

            for output_extension in output_extensions {
                let start = dump_start(format!("{}:@transpile using {}", input_path.as_os_str().to_string_lossy(), output_extension).as_str());
                match transpile_target(base_filename, base_output_path, &config, &mut runtime, &module, output_extension) {
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

fn transpile_target(base_filename: &str, base_output_path: &Path, config: &transpiler::Config, mut runtime: &mut Box<Runtime>, module: &Box<Module>, output_extension: &str) -> RResult<Vec<PathBuf>> {
    let mut context = match output_extension {
        "py" => transpiler::python::create_context(&runtime),
        output_extension => panic!("File type not supported: {}", output_extension)
    };

    let transpiler = interpreter::run::transpile(&module, &mut runtime)?;

    let file_map = context.make_files(base_filename, runtime, transpiler, config)?;
    let output_files = file_map.into_iter().map(|(filename, content)| {
        let file_path = base_output_path.join(filename);
        let mut f = File::create(file_path.clone()).expect("Unable to create file");
        let f: &mut (dyn Write) = &mut f;
        write!(f, "{}", content).expect("Error writing file");
        file_path
    }).collect_vec();
    Ok(output_files)
}
