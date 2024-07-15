use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};

use display_with_options::{DebugWithOptions, DisplayWithOptions};

pub use head::{FunctionHead, FunctionType};
pub use interface::{FunctionInterface, Parameter, ParameterKey};
pub use overload::FunctionOverload;
pub use representation::{FunctionCallExplicity, FunctionRepresentation, FunctionTargetType};
pub use function_binding::{FunctionBinding, resolve_binding, resolve_fulfillment};

mod interface;
mod head;
mod representation;
mod overload;
mod function_binding;
