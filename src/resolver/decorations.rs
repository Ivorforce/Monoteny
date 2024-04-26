use std::rc::Rc;

use itertools::Itertools;
use uuid::Uuid;

use crate::error::{RResult, RuntimeError, TryCollectMany};
use crate::resolver::grammar::{Pattern, PatternPart};
use crate::resolver::scopes;
use crate::parser::ast;
use crate::program::functions::{FunctionHead, ParameterKey};
use crate::util::position::Positioned;

pub fn try_parse_pattern(decoration: &ast::Expression, function: Rc<FunctionHead>, scope: &scopes::Scope) -> RResult<Rc<Pattern>> {
    let parameters = function.interface.parameters.iter().map(|p| p.internal_name.clone()).collect_vec();

    return match &decoration.iter().map(|a| a.as_ref()).collect_vec()[..] {
        [
            Positioned { position: p1, value: ast::Term::Identifier(i)},
            Positioned { position: p2, value: ast::Term::Struct(call_struct)}
        ] => {
            if i != "pattern" {
                return Err(RuntimeError::error("Unrecognized decoration.").to_array())
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
                head: function,
            }))
        }
        _ => Err(RuntimeError::error("Unrecognized decoration.").to_array())
    }
}
