pub mod scopes;
pub mod imperative;
pub mod traits;
pub mod conformance;
pub mod global;
pub mod ambiguous;
pub mod precedence;
pub mod r#type;
pub mod interface;

pub use crate::linker::global::link_file;

use custom_error::custom_error;

custom_error!{pub LinkError
    LinkError{msg: String} = "Linker Error: {msg}",
    Ambiguous = "Ambiguous",
}
