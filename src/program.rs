use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use itertools::Itertools;
use global::{FunctionImplementation};
use crate::program::functions::FunctionPointer;
use crate::program::module::Module;
use crate::program::traits::Trait;

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

pub struct Program {
    pub module: Module,
    pub function_implementations: HashMap<Rc<FunctionPointer>, Rc<FunctionImplementation>>,
}

impl Program {
    pub fn find_annotated(&self, annotation: &str) -> Option<&Rc<FunctionImplementation>> {
        self.function_implementations.values()
            .find_or_first(|f| f.decorators.contains(&annotation.into()))
    }
}
