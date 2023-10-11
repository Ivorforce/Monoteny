pub mod scopes;
pub mod imperative;
pub mod traits;
pub mod conformance;
pub mod global;
pub mod ambiguous;
pub mod precedence;
pub mod type_factory;
pub mod interface;
pub mod precedence_order;

pub use crate::linker::global::link_file;
