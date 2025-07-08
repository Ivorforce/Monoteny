use crate::cli::logging::dump_failure;
use clap::Command;
use std::process::ExitCode;

pub mod run;
pub mod expression;
pub mod check;
pub mod transpile;
pub mod logging;

pub fn make_command() -> Command {
    Command::new("monoteny")
        .about("A cli implementation for the monoteny language.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(run::make_command())
        .subcommand(expression::make_command())
        .subcommand(check::make_command())
        .subcommand(transpile::make_command())
}

pub fn run_command() -> ExitCode {
    let matches = make_command().get_matches();

    let result = match matches.subcommand() {
        Some(("run", sub_matches)) => run::run(sub_matches),
        Some(("expression", sub_matches)) => expression::run(sub_matches),
        Some(("check", sub_matches)) => check::run(sub_matches),
        Some(("transpile", sub_matches)) => transpile::run(sub_matches),
        _ => panic!("Unsupported action."),
    };

    result.unwrap_or_else(|e| dump_failure(e))
}
