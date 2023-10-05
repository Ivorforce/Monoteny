use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct RuntimeError {
    pub msg: String,
}

impl RuntimeError {
    pub fn with_cause(&self, cause: &str) -> RuntimeError {
        RuntimeError {
            msg: format!("{}\n{}", cause, self.msg)
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
