use std::time::Instant;
use std::process::ExitCode;
use colored::Colorize;
use crate::error;
use crate::error::{print_errors, RResult, RuntimeError};

pub fn dump_start(name: &str) -> Instant {
    println!("{} {}", "Running".green().bold(), name);
    Instant::now()
}

pub fn dump_result<V>(start: Instant, result: RResult<V>) -> ExitCode {
    match result {
        Ok(_) => dump_success(start,),
        Err(e) => dump_failure(e),
    }
}

pub fn dump_named_failure(name: &str, err: Vec<RuntimeError>) -> ExitCode {
    print_errors(&err);
    println!("\n{} on {}: {} error(s)", "Failure".red().bold(), name, err.len());
    ExitCode::FAILURE
}

pub fn dump_failure(err: Vec<RuntimeError>) -> ExitCode {
    print_errors(&err);
    println!("\n{}: {} error(s)", "Failure".red().bold(), err.len());
    ExitCode::FAILURE
}

pub fn dump_success(start: Instant) -> ExitCode {
    println!("{} in {:.2}s", "Finished".green().bold(), start.elapsed().as_secs_f32());
    ExitCode::SUCCESS
}
