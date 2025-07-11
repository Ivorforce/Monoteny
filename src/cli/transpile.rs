use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{arg, ArgAction, ArgMatches, Command};
use itertools::Itertools;

use crate::cli::logging::{dump_failure, dump_start, dump_success};
use crate::error::{RResult, RuntimeError};
use crate::interpreter::runtime::Runtime;
use crate::program::module::{module_name, Module};
use crate::transpiler::LanguageContext;
use crate::util::file_writer::write_file_safe;
use crate::{interpreter, transpiler};

pub fn make_command() -> Command {
    Command::new("transpile")
        .about("Transpile a file into another language.")
        .arg_required_else_help(true)
        .arg(arg!(<INPUT> "file to transpile").value_parser(clap::value_parser!(PathBuf)).long("input").short('i'))
        .arg(arg!(<OUTPUT> "output file path").required(false).value_parser(clap::value_parser!(PathBuf)).long("output").short('o'))
        .arg(arg!(<ALL> "use all available transpilers").required(false).action(ArgAction::SetTrue).long("all"))
        .arg(arg!(<NOREFACTOR> "don't use ANY refactoring").required(false).action(ArgAction::SetTrue).long("norefactor"))
        .arg(arg!(<NOFOLD> "don't use constant folding").required(false).action(ArgAction::SetTrue).long("nofold"))
        .arg(arg!(<NOINLINE> "don't use inlining").required(false).action(ArgAction::SetTrue).long("noinline"))
        .arg(arg!(<NOTRIMLOCALS> "don't trim unused locals code").required(false).action(ArgAction::SetTrue).long("notrimlocals"))
}

pub fn run(args: &ArgMatches) -> RResult<ExitCode> {
    let input_path = args.get_one::<PathBuf>("INPUT").unwrap();
    let output_path_proto = match args.contains_id("OUTPUT") {
        true => args.get_one::<PathBuf>("OUTPUT").unwrap().clone(),
        false => input_path.with_extension(""),
    };
    let base_filename = output_path_proto.file_name().and_then(OsStr::to_str).unwrap();
    let base_output_path = output_path_proto.parent().unwrap();

    let can_refactor = !args.get_flag("NOREFACTOR");
    let config = transpiler::Config {
        should_constant_fold: can_refactor && !args.get_flag("NOFOLD"),
        should_monomorphize: true, // TODO Cannot do without it for now
        should_inline: can_refactor && !args.get_flag("NOINLINE"),
        should_trim_locals: can_refactor && !args.get_flag("NOTRIMLOCALS"),
    };
    let should_output_all = args.get_flag("ALL");

    let output_extensions: Vec<&str> = match should_output_all {
        true => vec!["py"],
        false => vec![output_path_proto.extension().and_then(OsStr::to_str).ok_or_else(|| vec![RuntimeError::error("Error: must provide either output path or --all")])?]
    };

    let mut runtime = Runtime::new()?;
    runtime.add_common_repository();

    let module = runtime.load_file_as_module(input_path, module_name("main"))?;

    let mut error_count = 0;

    for output_extension in output_extensions {
        let start = dump_start(format!("{}:transpile! using {}", input_path.as_os_str().to_string_lossy(), output_extension).as_str());
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
    
    Ok(ExitCode::from(error_count))
}

fn create_context(runtime: &Runtime, extension: &str) -> Box<dyn LanguageContext> {
    match extension {
        "py" => Box::new(transpiler::python::Context::new(runtime)),
        _ => panic!("File type not supported: {}", extension)
    }
}

fn transpile_target(base_filename: &str, base_output_path: &Path, config: &transpiler::Config, mut runtime: &mut Box<Runtime>, module: &Box<Module>, output_extension: &str) -> RResult<Vec<PathBuf>> {
    let context = create_context(&runtime, output_extension);
    let transpiler = interpreter::run::transpile(&module, runtime)?;
    let file_map = transpiler::transpile(transpiler, runtime, context.as_ref(), config, base_filename)?;

    let output_files = file_map.into_iter().map(|(filename, content)| {
        write_file_safe(base_output_path, &filename, &content)
    }).collect_vec();
    Ok(output_files)
}
