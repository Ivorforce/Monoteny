use std::collections::HashMap;
use std::rc::Rc;
use itertools::Itertools;
use global::{FunctionImplementation};
use crate::program::functions::FunctionPointer;
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
    pub module: Module,
    pub function_implementations: HashMap<Rc<FunctionPointer>, Rc<FunctionImplementation>>,
}

pub fn find_annotated<'a, I>(iterator: I, annotation: &str) -> Option<&'a Rc<FunctionImplementation>> where I: Iterator<Item=&'a Rc<FunctionImplementation>> {
    let results = iterator.filter(|f| f.decorators.contains(&annotation.into()))
        .collect_vec();
    if results.len() > 1 {
        panic!()
    }
    results.first().copied()
}
