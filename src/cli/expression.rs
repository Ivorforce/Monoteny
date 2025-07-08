use std::process::ExitCode;

use clap::{arg, ArgMatches, Command};

use crate::error::RResult;
use crate::interpreter::runtime::Runtime;
use crate::interpreter;

pub fn make_command() -> Command {
    Command::new("expression")
        .about("Run an expression using the interpreter.")
        .arg_required_else_help(true)
        .arg(arg!(<EXPRESSION> "expression to run").value_parser(clap::value_parser!(String)))
}

pub fn run(args: &ArgMatches) -> RResult<ExitCode> {
    let expression = args.get_one::<String>("EXPRESSION").unwrap();
    let full_expression = format!("use!(module!(\"common\")); def main! :: write_line({});", expression);

    let mut runtime = Runtime::new()?;
    runtime.add_common_repository();

    let module = runtime.load_text_as_module(&full_expression, vec!["dynamic".to_string()])?;

    interpreter::run::main(&module, &mut runtime)?;

    Ok(ExitCode::SUCCESS)
}
