use itertools::Itertools;
use crate::error::{ErrInRange, RResult, RuntimeError};
use crate::linker::interface::ParameterKey;
use crate::parser::ast;
use crate::util::position::Positioned;

pub fn plain_parameter<'a>(cause: &str, args: &'a Vec<ast::StructArgument>) -> RResult<&'a ast::Expression> {
    let body = args.iter().exactly_one()
        .map_err(|_| RuntimeError::new(format!("{} needs exactly one parameter.", cause)))?;
    if body.key != ParameterKey::Positional {
        return Err(RuntimeError::new(format!("{} needs exactly one parameter.", cause)));
    }
    if body.type_declaration.is_some() {
        return Err(RuntimeError::new(format!("{} needs exactly one parameter.", cause)));
    }

    Ok(&body.value)
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
