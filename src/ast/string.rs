use std::fmt::{Display, Formatter};

use crate::ast::Struct;

#[derive(PartialEq, Eq, Clone)]
pub enum StringPart {
    Literal(String),
    Object(Box<Struct>),
}

impl Display for StringPart {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StringPart::Literal(s) => write!(f, "{}", s),
            StringPart::Object(struct_) => write!(f, "{}", struct_),
        }
    }
}
