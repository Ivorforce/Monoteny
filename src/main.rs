#[macro_use]
extern crate lalrpop_util;
extern crate core;

lalrpop_mod!(pub tenlang_grammar);
pub mod parser;
pub mod linker;
pub mod languages;
pub mod program;

use std::ffi::OsStr;
use std::fs::File;
use std::path::PathBuf;

use clap::{arg, Command};
use crate::parser::parse_program;

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
                .arg(arg!(<TREE> "dump the parse tree to stdout").required(false).takes_value(false).long("tree"))
                .arg(arg!(<PATH> ... "files to check").value_parser(clap::value_parser!(PathBuf)))
        )
        .subcommand(
            Command::new("transpile")
                .about("Transpile a file into another language.")
                .arg_required_else_help(true)
                .arg(arg!(<INPUT> "file to transpile").value_parser(clap::value_parser!(PathBuf)).long("input").short('i'))
                .arg(arg!(<OUTPUT> "output file path").value_parser(clap::value_parser!(PathBuf)).long("output").short('o'))
                .arg(arg!(<ALL> "output using all available transpilers").required(false).takes_value(false).long("all"))
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
            let should_output_tree = sub_matches.is_present("TREE");

            for path in paths {
                println!("Checking {:?}...", path);

                let content = std::fs::read_to_string(&path)
                    .expect("could not read file");

                let builtins = program::builtins::create_builtins();
                let builtin_variable_scope = builtins.global_constants.as_global_scope();

                let syntax_tree = parse_program(&content, &builtins.parser_constants, &builtins);

                if should_output_tree {
                    println!("{:?}", syntax_tree);
                }
            }

            println!("All files are valid .tenlang!");
        },
        Some(("transpile", sub_matches)) => {
            let input_path = sub_matches.get_one::<PathBuf>("INPUT").unwrap();
            let output_path = sub_matches.get_one::<PathBuf>("OUTPUT").unwrap();
            let should_output_all = sub_matches.is_present("ALL");

            let output_extensions: Vec<&str> = match should_output_all {
                true => vec!["py", "cpp"],
                false => vec![output_path.extension().and_then(OsStr::to_str).unwrap()]
            };

            let content = std::fs::read_to_string(&input_path)
                .expect("could not read file");

            let builtins = program::builtins::create_builtins();
            let builtin_variable_scope = builtins.global_constants.as_global_scope();

            let syntax_tree = parse_program(&content, &builtins.parser_constants, &builtins);
            let computation_tree = linker::link_program(syntax_tree, &builtin_variable_scope, &builtins);

            for output_extension in output_extensions {
                match output_extension {
                    "py" => {
                        let python_path = output_path.with_extension("py");
                        let mut f = File::create(python_path.clone()).expect("Unable to create file");

                        languages::python::transpile_program(
                            &mut f,
                            &computation_tree,
                            &builtins
                        ).expect("Error when writing to file");

                        println!("{:?}", python_path);
                    },
                    "cpp" => {
                        let header_path = output_path.with_extension("hpp");
                        let source_path = output_path.with_extension("cpp");

                        let mut f_header = File::create(header_path.clone()).expect("Unable to create file");
                        let mut f_source = File::create(source_path.clone()).expect("Unable to create file");

                        languages::cpp::transpile_program(
                            &computation_tree,
                            &mut f_header,
                            &mut f_source
                        ).expect("Error when writing to file");

                        println!("{:?}", header_path);
                        println!("{:?}", source_path);
                    },
                    _ => unreachable!()
                };
            }
        },
        _ => unreachable!(),
    }
}
