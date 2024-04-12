use std::path::PathBuf;
use std::process::ExitCode;

use clap::{arg, ArgMatches, Command};

use crate::error::{dump_named_failure, dump_start, dump_success, RResult};
use crate::interpreter::Runtime;
use crate::program::module::module_name;

pub fn make_command() -> Command<'static> {
    Command::new("check")
        .about("Parse files to check for validity.")
        .arg_required_else_help(true)
        .arg(arg!(<PATH> ... "files to check").value_parser(clap::value_parser!(PathBuf)))
}

pub fn run(args: &ArgMatches) -> RResult<ExitCode> {
    let paths = args
        .get_many::<PathBuf>("PATH")
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    let start = dump_start(format!("check for {} file(s)", paths.len()).as_str());

    let mut runtime = Runtime::new()?;
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

    Ok(ExitCode::from(error_count))
}
