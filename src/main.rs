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
pub mod generic_unfolding;
pub mod integration_tests;

use std::ffi::OsStr;
use std::fs::File;
use std::path::PathBuf;
use std::rc::Rc;

use clap::{arg, Command};
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
                .arg(arg!(<NOFOLD> "don't use constant folding to shorten the code at compile time").required(false).takes_value(false).long("nofold"))
        )
}

fn main() -> Result<(), LinkError> {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("run", sub_matches)) => {
            let path = sub_matches.get_one::<PathBuf>("PATH").unwrap();

            let builtins = program::builtins::create_builtins();
            let builtin_variable_scope = builtins.create_scope();

            let content = std::fs::read_to_string(&path)
                .expect("could not read file");

            let syntax_tree = parser::parse_program(&content);

            let computation_tree = linker::link_program(syntax_tree, &builtin_variable_scope, &builtins)?;
            interpreter::run::main(&computation_tree, &builtins);
        },
        Some(("check", sub_matches)) => {
            let paths = sub_matches
                .get_many::<PathBuf>("PATH")
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            let should_output_tree = sub_matches.is_present("TREE");

            let builtins = program::builtins::create_builtins();
            let builtin_variable_scope = &builtins.create_scope();

            for path in paths {
                println!("Checking {:?}...", path);

                let content = std::fs::read_to_string(&path)
                    .expect("could not read file");

                let syntax_tree = parser::parse_program(&content);

                if should_output_tree {
                    println!("{:?}", &syntax_tree);
                }

                let _ = linker::link_program(syntax_tree, &builtin_variable_scope, &builtins)?;
            }

            println!("All files are valid .monoteny!");
        },
        Some(("transpile", sub_matches)) => {
            let input_path = sub_matches.get_one::<PathBuf>("INPUT").unwrap();
            let output_path = sub_matches.get_one::<PathBuf>("OUTPUT").unwrap();
            let should_output_all = sub_matches.is_present("ALL");
            let should_constant_fold = !sub_matches.is_present("NOFOLD");

            let output_extensions: Vec<&str> = match should_output_all {
                true => vec!["py"],
                false => vec![output_path.extension().and_then(OsStr::to_str).unwrap()]
            };

            let content = std::fs::read_to_string(&input_path)
                .expect("could not read file");

            let builtins = program::builtins::create_builtins();

            let syntax_tree = parser::parse_program(&content);

            let builtin_variable_scope = &builtins.create_scope();
            let computation_tree = linker::link_program(syntax_tree, &builtin_variable_scope, &builtins)?;

            for output_extension in output_extensions {
                match output_extension {
                    "py" => {
                        let python_path = output_path.with_extension("py");
                        let mut f = File::create(python_path.clone()).expect("Unable to create file");

                        transpiler::python::transpile_program(
                            &mut f,
                            &computation_tree,
                            Rc::clone(&builtins)
                        ).expect("Error when writing to file");

                        println!("{:?}", python_path);
                    },
                    "cpp" => {
                        let header_path = output_path.with_extension("hpp");
                        let source_path = output_path.with_extension("cpp");

                        let mut f_header = File::create(header_path.clone()).expect("Unable to create file");
                        let mut f_source = File::create(source_path.clone()).expect("Unable to create file");

                        transpiler::cpp::transpile_program(
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
        _ => {
            panic!("Unsupported action.")
        },
    }

    Ok(())
}
