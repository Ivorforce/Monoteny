use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use builtins::TenLangBuiltins;
use types::{FunctionInterface, PassedArgumentType, Type, Variable};
use crate::linker::computation_tree::PassedArgument;

pub mod builtins;
pub mod types;
pub mod primitives;
pub mod scopes;
