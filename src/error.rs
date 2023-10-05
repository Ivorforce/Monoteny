use std::cmp::max;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Range;
use std::path::PathBuf;
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
                    write!(f, "--> {}:{}:{}\n", path_str, line_start + 1, range.start - line_idxs[max(0, line_start - 1)].0)?;
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
