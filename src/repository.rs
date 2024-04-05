use std::collections::HashMap;
use std::path::PathBuf;
use itertools::Itertools;
use crate::error::{RResult, RuntimeError};
use crate::program::module::ModuleName;

pub struct Repository {
    pub entries: HashMap<String, PathBuf>,
}

impl Repository {
    pub fn new() -> Box<Repository> {
        Box::new(Repository {
            entries: Default::default(),
        })
    }

    pub fn add(&mut self, name: &str, path: PathBuf) {
        self.entries.insert(name.to_string(), path);
    }

    pub fn resolve_module_path(&self, name: &ModuleName) -> RResult<PathBuf> {
        let Some(first_part) = name.first() else {
            return Err(RuntimeError::new(format!("Module name is empty...")));
        };

        let Some(base_path) = self.entries.get(first_part) else {
            return Err(RuntimeError::new(format!("Module not in repository: {}", first_part)));
        };

        Ok(base_path.join(PathBuf::from(format!("{}.monoteny", name.join("/")))))
    }
}
