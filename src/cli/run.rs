use std::path::PathBuf;
use std::process::ExitCode;

use clap::{arg, ArgMatches, Command};

use crate::error::RResult;
use crate::interpreter;
use crate::interpreter::runtime::Runtime;
use crate::program::module::module_name;

pub fn make_command() -> Command {
    Command::new("run")
        .about("Run a file using the interpreter.")
        .arg_required_else_help(true)
        .arg(arg!(<PATH> "file to run").value_parser(clap::value_parser!(PathBuf)))
}

pub fn run(args: &ArgMatches) -> RResult<ExitCode> {
    let input_path = args.get_one::<PathBuf>("PATH").unwrap();

    let mut runtime = Runtime::new()?;
    runtime.add_common_repository();

    let module = runtime.load_file_as_module(input_path, module_name("main"))?;

    interpreter::run::main(&module, &mut runtime)?;

    Ok(ExitCode::SUCCESS)
}
