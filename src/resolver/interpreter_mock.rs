use itertools::Itertools;

use crate::error::{ErrInRange, RResult, RuntimeError};
use crate::parser::ast;
use crate::program::functions::ParameterKey;
use crate::util::position::Positioned;

pub fn plain_parameter<'a>(cause: &str, struct_: &'a ast::Struct) -> RResult<&'a ast::Expression> {
    let body = struct_.arguments.iter().exactly_one()
        .map_err(|_| RuntimeError::new(format!("{} needs exactly one parameter.", cause)))?;
    if body.value.key != ParameterKey::Positional {
        return Err(RuntimeError::new(format!("{} needs exactly one parameter.", cause)));
    }
    if body.value.type_declaration.is_some() {
        return Err(RuntimeError::new(format!("{} needs exactly one parameter.", cause)));
    }

    Ok(&body.value.value)
}

pub fn plain_string_literal<'a>(cause: &str, literal: &'a Vec<Box<Positioned<ast::StringPart>>>) -> RResult<&'a str> {
    match &literal[..] {
        [part] => {
            match &part.value {
                ast::StringPart::Literal(literal) => {
                    Ok(literal)
                }
                _ => Err(RuntimeError::new(format!("{} needs a plain string literal.", cause))).err_in_range(&part.position),
            }
        }
        _ => Err(RuntimeError::new(format!("{} needs a plain string literal.", cause))),
    }
}
