use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};

use display_with_options::{DebugWithOptions, DisplayWithOptions};

pub use head::{FunctionHead, FunctionType};
pub use interface::{FunctionInterface, Parameter, ParameterKey};

mod interface;
mod head;

