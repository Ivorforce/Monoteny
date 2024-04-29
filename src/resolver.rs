pub use crate::resolver::global::resolve_file;

pub mod scopes;
pub mod imperative;
pub mod traits;
pub mod conformance;
pub mod global;
pub mod ambiguous;
pub mod type_factory;
pub mod interface;
pub mod fields;
pub mod imports;
pub mod interpreter_mock;
pub mod referencible;
pub mod structs;
pub mod decorations;
pub mod precedence_order;
pub mod function;

