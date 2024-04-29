use std::rc::Rc;

use itertools::Itertools;
use uuid::Uuid;

use crate::ast;
use crate::error::{RResult, RuntimeError, TryCollectMany};
use crate::parser::expressions;
use crate::parser::grammar::{Pattern, PatternPart};
use crate::program::functions::{FunctionHead, ParameterKey};
use crate::resolver::scopes;
use crate::util::position::Positioned;

pub fn try_parse_pattern(decoration: &ast::Expression, function: Rc<FunctionHead>, scope: &scopes::Scope) -> RResult<Rc<Pattern<Rc<FunctionHead>>>> {
    let parameters = function.interface.parameters.iter().map(|p| p.internal_name.clone()).collect_vec();

    let parsed = expressions::parse(decoration, &scope.grammar)?;

    let expressions::Value::FunctionCall(target, call_struct) = &parsed.value else {
        return Err(RuntimeError::error("Unrecognized decoration.").to_array());
    };

    let expressions::Value::Identifier(decoration_name) = &target.value else {
        return Err(RuntimeError::error("Unrecognized decoration.").to_array());
    };

    if decoration_name.as_str() != "pattern" {
        return Err(RuntimeError::error("Unrecognized decoration.").to_array());
    }

    let [a, b] = &call_struct.arguments[..] else {
        return Err(RuntimeError::error("pattern decoration needs two arguments.").to_array())
    };

    if a.value.key != ParameterKey::Positional || a.value.type_declaration.is_some() ||
        b.value.key != ParameterKey::Positional || b.value.type_declaration.is_some() {
        return Err(RuntimeError::error("pattern decoration arguments are faulty.").to_array())
    }

    let precedence_group = match &b.value.value.iter().map(|p| p.as_ref()).collect_vec()[..] {
        [Positioned { position, value: ast::Term::Identifier(precedence) }] =>
            scope.resolve_precedence_group(&precedence)?,
        _ => return Err(RuntimeError::error("Second argument to pattern needs to be a precedence name.").to_array())
    };

    let parts: Vec<Box<PatternPart>> = a.value.value.iter()
        .map(|pterm| {
            match &pterm.value {
                ast::Term::Identifier(i) => {
                    Ok(Box::new(parameters.iter()
                        .position(|p| p == i)
                        .map(|p| PatternPart::Parameter(p))
                        .unwrap_or(PatternPart::Keyword(i.clone()))))
                },
                _ => Err(RuntimeError::error("Bad pattern.").to_array()),
            }
        })
        .try_collect_many()?;

    Ok(Rc::new(Pattern {
        id: Uuid::new_v4(),
        precedence_group,
        parts,
        function: function,
    }))
}
