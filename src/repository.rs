use std::collections::HashMap;
use std::path::PathBuf;
use crate::program::module::ModuleName;

pub enum Loader {
    Path(PathBuf),
    Intrinsic(HashMap<ModuleName, &'static str>)
}

pub struct Repository {
    pub entries: HashMap<String, Loader>,
}

impl Repository {
    pub fn new() -> Box<Repository> {
        Box::new(Repository {
            entries: Default::default(),
        })
    }

    pub fn add(&mut self, name: &str, loader: Loader) {
        self.entries.insert(name.to_string(), loader);
    }
}
