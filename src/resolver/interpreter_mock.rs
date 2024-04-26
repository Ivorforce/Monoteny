use itertools::Itertools;

use crate::error::{ErrInRange, RResult, RuntimeError};
use crate::parser::ast;
use crate::program::functions::ParameterKey;
use crate::util::position::Positioned;

pub fn plain_parameter<'a>(cause: &str, struct_: &'a ast::Struct) -> RResult<&'a ast::Expression> {
    let body = struct_.arguments.iter().exactly_one()
        .map_err(|_| RuntimeError::error(format!("{} needs exactly one parameter.", cause).as_str()).to_array())?;

    if body.value.key != ParameterKey::Positional {
        return Err(RuntimeError::error(format!("{} needs exactly one parameter.", cause).as_str()).to_array());
    }

    if body.value.type_declaration.is_some() {
        return Err(RuntimeError::error(format!("{} needs exactly one parameter.", cause).as_str()).to_array());
    }

    Ok(&body.value.value)
}

pub fn plain_string_literal<'a>(cause: &str, literal: &'a Vec<Box<Positioned<ast::StringPart>>>) -> RResult<&'a str> {
    let [part] = &literal[..] else {
        return Err(RuntimeError::error(format!("{} needs a plain string literal.", cause).as_str()).to_array());
    };

    let ast::StringPart::Literal(literal) = &part.value else {
        return Err(RuntimeError::error(format!("{} needs a plain string literal.", cause).as_str()).to_array()).err_in_range(&part.position);
    };

    Ok(literal)
}
