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
                    let line_end = line_idxs[..].binary_search_by(|x| x.0.cmp(&range.end)).unwrap_or_else(|e| e);
                    write!(f, "in file (l {}..{}, p {}..{}): {}\n", line_start + 1, line_end + 1, range.start, range.end, file.as_os_str().to_string_lossy())?;
                }
                else {
                    write!(f, "in file (p {}..{}): {}\n", range.start, range.end, file.as_os_str().to_string_lossy())?;
                }
            }
            else {
                write!(f, "in file: {}\n", file.as_os_str().to_string_lossy())?;
            }
        }
        else if let Some(range) = &self.range {
            write!(f, "in unknown file (p {}..{})", range.start, range.end)?;
        }

        Ok(())
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.position, self.msg)
    }
}
