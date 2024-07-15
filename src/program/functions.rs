use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};

use display_with_options::{DebugWithOptions, DisplayWithOptions};

pub use function_binding::{FunctionBinding, resolve_binding};
pub use head::{FunctionHead, FunctionType};
pub use implementation::FunctionImplementation;
pub use interface::{FunctionInterface, Parameter, ParameterKey};
pub use logic::{FunctionLogic, FunctionLogicDescriptor, PrimitiveOperation};
pub use overload::FunctionOverload;
pub use representation::{FunctionCallExplicity, FunctionRepresentation, FunctionTargetType};

mod interface;
mod head;
mod representation;
mod overload;
mod function_binding;
mod logic;
mod implementation;
