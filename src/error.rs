use std::cmp::max;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Range;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;
use colored::Colorize;
use itertools::Itertools;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FilePosition {
    pub file: Option<PathBuf>,
    pub range: Option<Range<usize>>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct RuntimeError {
    pub position: FilePosition,
    pub msg: String,
}

pub type RResult<V> = Result<V, RuntimeError>;

impl RuntimeError {
    pub fn new(msg: String) -> RuntimeError {
        RuntimeError {
            position: FilePosition {
                file: None,
                range: None,
            },
            msg,
        }
    }

    pub fn in_range(&self, range: Range<usize>) -> RuntimeError {
        if self.position.range.is_some() {
            return self.clone();
        }

        RuntimeError {
            position: FilePosition {
                file: self.position.file.clone(),
                range: Some(range)
            },
            msg: self.msg.clone(),
        }
    }

    pub fn in_file(&self, path: PathBuf) -> RuntimeError {
        RuntimeError {
            position: FilePosition {
                file: Some(path),
                range: self.position.range.clone()
            },
            msg: self.msg.clone(),
        }
    }
}

impl Display for FilePosition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(file) = &self.file {
            if let Some(range) = &self.range {
                if let Ok(contents) = std::fs::read_to_string(&file) {
                    let mut line_idxs = contents.match_indices("\n").collect_vec();
                    let line_start = line_idxs[..].binary_search_by(|x| x.0.cmp(&range.start)).unwrap_or_else(|e| e);
                    let path_str = file.as_os_str().to_string_lossy();
                    // For some reason, in line idx is always one too much.
                    // Lines are correct but the unwritten rule is lines start at idx 1.
                    write!(f, "--> {}:{}:{}\n", path_str, line_start + 1, range.start - line_idxs[max(0, line_start - 1)].0 - 1)?;
                }
                else {
                    write!(f, "in file ({}..{}) -> {}\n", range.start, range.end, file.as_os_str().to_string_lossy())?;
                }
            }
            else {
                write!(f, "--> {}\n", file.as_os_str().to_string_lossy())?;
            }
        }
        else if let Some(range) = &self.range {
            write!(f, "in unknown file ({}..{})", range.start, range.end)?;
        }

        Ok(())
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.position, self.msg)
    }
}

pub trait ErrInRange<R> {
    fn err_in_range(self, range: &Range<usize>) -> R;
}

impl<V> ErrInRange<RResult<V>> for RResult<V> {
    fn err_in_range(self, range: &Range<usize>) -> RResult<V> {
        self.map_err(|e| e.in_range(range.clone()))
    }
}

pub fn dump_start(name: &str) -> Instant {
    println!("{} {}", "Running".green().bold(), name);
    Instant::now()
}

pub fn dump_result<V>(start: Instant, result: Result<V, Vec<RuntimeError>>) -> ExitCode {
    match result {
        Ok(_) => dump_success(start,),
        Err(e) => dump_failure(e),
    }
}

pub fn dump_unexpected_failure(name: &str, err: Vec<RuntimeError>) -> ExitCode {
    println!("{} on {} ({} error(s)):\n\n{}", "Failure".red().bold(), name, err.len(), format_errors(&err));
    ExitCode::FAILURE
}

pub fn dump_failure(err: Vec<RuntimeError>) -> ExitCode {
    println!("{} ({} error(s)):\n\n{}", "Failure".red().bold(), err.len(), format_errors(&err));
    ExitCode::FAILURE
}

pub fn dump_success(start: Instant) -> ExitCode {
    println!("{} in {:.2}s", "Finished".green().bold(), start.elapsed().as_secs_f32());
    ExitCode::SUCCESS
}

pub fn format_errors(errors: &Vec<RuntimeError>) -> String {
    errors.into_iter().map(|e| e.to_string()).join("\n\n")
}
