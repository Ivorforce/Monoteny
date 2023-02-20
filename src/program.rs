use std::collections::HashSet;
use std::rc::Rc;
use global::{FunctionImplementation, GlobalStatement};
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
    pub functions: HashSet<Rc<FunctionImplementation>>,
    pub traits: HashSet<Rc<Trait>>,

    pub global_statements: Vec<GlobalStatement>,
    pub main_function: Option<Rc<FunctionImplementation>>,
}
