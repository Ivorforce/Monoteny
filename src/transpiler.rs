use std::collections::HashMap;
use std::ops::DerefMut;
use std::rc::Rc;
use itertools::Itertools;
use crate::error::RResult;
use crate::interpreter::Runtime;
use crate::program::functions::FunctionHead;
use crate::program::global::FunctionImplementation;

pub mod python;
pub mod namespaces;
pub mod structs;

pub struct Config {
    pub should_constant_fold: bool,
    pub should_monomorphize: bool,
    pub should_inline: bool,
    pub should_trim_locals: bool,
}

pub enum TranspiledArtifact {
    Function(Box<FunctionImplementation>)
}

pub struct Transpiler {
    // In the future, this should all be accessible by monoteny code itself - including the context.
    pub main_function: Option<Rc<FunctionHead>>,
    pub exported_artifacts: Vec<TranspiledArtifact>,
}

pub trait LanguageContext {
    fn make_files(&self, filename: &str, runtime: &mut Runtime, transpiler: Box<Transpiler>, config: &Config) -> RResult<HashMap<String, String>>;
}
