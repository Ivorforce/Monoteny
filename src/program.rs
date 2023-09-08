use std::rc::Rc;
use itertools::Itertools;
use global::{FunctionImplementation};
use crate::interpreter::RuntimeError;
use crate::program::module::Module;

pub mod allocation;
pub mod builtins;
pub mod computation_tree;
pub mod functions;
pub mod generics;
pub mod global;
pub mod primitives;
pub mod traits;
pub mod types;
pub mod module;
pub mod calls;

pub struct Program {
    pub module: Rc<Module>,
}

pub fn find_one_annotated_function<'a, I>(iterator: I, annotation: &str) -> Result<&'a Rc<FunctionImplementation>, RuntimeError> where I: Iterator<Item=&'a Rc<FunctionImplementation>> {
    let matches = iterator
        .filter(|f| f.decorators.contains(&annotation.into()))
        .collect_vec();

    match matches.len() {
        0 => Err(RuntimeError::RuntimeError { msg: format!("No function found annotated with @{}", annotation) }),
        1 => Ok(matches[0]),
        _ => Err(RuntimeError::RuntimeError { msg: format!("Too many functions found annotated with @{} ({}): {:?}", annotation, matches.len(), matches.iter().map(|x| &x.head).collect_vec()) }),
    }
}
