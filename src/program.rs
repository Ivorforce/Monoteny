use std::rc::Rc;
use itertools::Itertools;
use global::{FunctionImplementation};
use crate::interpreter::InterpreterError;
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
pub mod r#struct;
