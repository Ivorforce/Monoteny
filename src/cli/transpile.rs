use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use clap::{arg, ArgMatches, Command};
use itertools::Itertools;
use crate::error::{dump_failure, dump_start, dump_success, RResult};
use crate::interpreter::Runtime;
use crate::program::module::{Module, module_name};
use crate::transpiler;
use crate::transpiler::LanguageContext;
use crate::util::file_writer::write_file_safe;

pub fn make_command() -> Command<'static> {
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
}

pub fn run(args: &ArgMatches) -> RResult<ExitCode> {
    let input_path = args.get_one::<PathBuf>("INPUT").unwrap();
    let output_path_proto = match args.contains_id("OUTPUT") {
        true => args.get_one::<PathBuf>("OUTPUT").unwrap().clone(),
        false => input_path.with_extension(""),
    };
    let base_filename = output_path_proto.file_name().and_then(OsStr::to_str).unwrap();
    let base_output_path = output_path_proto.parent().unwrap();

    let can_refactor = !args.is_present("NOREFACTOR");
    let config = transpiler::Config {
        should_constant_fold: can_refactor && !args.is_present("NOFOLD"),
        should_monomorphize: true, // TODO Cannot do without it for now
        should_inline: can_refactor && !args.is_present("NOINLINE"),
        should_trim_locals: can_refactor && !args.is_present("NOTRIMLOCALS"),
    };
    let should_output_all = args.is_present("ALL");

    let output_extensions: Vec<&str> = match should_output_all {
        true => vec!["py"],
        false => vec![output_path_proto.extension().and_then(OsStr::to_str).unwrap()]
    };

    let mut runtime = Runtime::new()?;
    runtime.repository.add("common", PathBuf::from("monoteny"));

    let module = runtime.load_file(input_path, module_name("main"))?;

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
    let file_map = transpiler::transpile(module, runtime, context.as_ref(), config, base_filename)?;

    let output_files = file_map.into_iter().map(|(filename, content)| {
        write_file_safe(base_output_path, &filename, &content)
    }).collect_vec();
    Ok(output_files)
}
