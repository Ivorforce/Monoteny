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

pub type RResult<V> = Result<V, Vec<RuntimeError>>;

impl RuntimeError {
    pub fn new(msg: String) -> Vec<RuntimeError> {
        vec![RuntimeError {
            position: FilePosition {
                file: None,
                range: None,
            },
            msg,
        }]
    }

    pub fn new_in_range(msg: String, range: Range<usize>) -> Vec<RuntimeError> {
        vec![RuntimeError {
            position: FilePosition {
                file: None,
                range: None,
            },
            msg,
        }.in_range(range)]
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
        if self.position.file.is_some() {
            return self.clone();
        }

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
                    let mut lines = contents.match_indices("\n").collect_vec();
                    // FIXME Insert first line because it's not a \n match... but split doesn't return indices
                    lines.insert(0, (0, lines.get(0).map(|e| e.1).unwrap_or("")));

                    let line_start = lines[..].binary_search_by(|x| x.0.cmp(&range.start)).unwrap_or_else(|e| e);
                    let path_str = file.as_os_str().to_string_lossy();
                    // For some reason, in line idx is always one too much.
                    // Lines are correct but the unwritten rule is lines start at idx 1.
                    write!(f, "\n  --> {}:{}:{}", path_str, max(line_start, 1), range.start - lines[max(line_start, 1) - 1].0)?;
                }
                else {
                    write!(f, "\n  --> in {} (range {}..{})", file.as_os_str().to_string_lossy(), range.start, range.end)?;
                }
            }
            else {
                write!(f, "\n  --> {}", file.as_os_str().to_string_lossy())?;
            }
        }
        else if let Some(range) = &self.range {
            write!(f, "  --> in unknown file (range {}..{})", range.start, range.end)?;
        }

        Ok(())
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}{}", "Error".red().bold(), self.msg, self.position)
    }
}

pub trait ErrInRange<R> {
    fn err_in_range(self, range: &Range<usize>) -> R;
}

impl<V> ErrInRange<RResult<V>> for RResult<V> {
    fn err_in_range(self, range: &Range<usize>) -> RResult<V> {
        self.map_err(|e| e.iter().map(|e| e.in_range(range.clone())).collect())
    }
}

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
    println!("{}", format_errors(&err));
    println!("\n{} on {}: {} error(s)", "Failure".red().bold(), name, err.len());
    ExitCode::FAILURE
}

pub fn dump_failure(err: Vec<RuntimeError>) -> ExitCode {
    println!("{}", format_errors(&err));
    println!("\n{}: {} error(s)", "Failure".red().bold(), err.len());
    ExitCode::FAILURE
}

pub fn dump_success(start: Instant) -> ExitCode {
    println!("{} in {:.2}s", "Finished".green().bold(), start.elapsed().as_secs_f32());
    ExitCode::SUCCESS
}

pub fn format_errors(errors: &Vec<RuntimeError>) -> String {
    errors.into_iter().map(|e| e.to_string()).join("\n\n")
}
