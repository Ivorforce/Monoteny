use std::rc::Rc;

use itertools::Itertools;
use uuid::Uuid;

use crate::error::{RResult, RuntimeError};
use crate::linker::grammar::{Pattern, PatternPart};
use crate::linker::scopes;
use crate::parser::ast;
use crate::program::functions::{FunctionHead, ParameterKey};
use crate::util::position::Positioned;

pub fn try_parse_pattern(decoration: &ast::Expression, function: Rc<FunctionHead>, scope: &scopes::Scope) -> RResult<Rc<Pattern>> {
    let parameters = function.interface.parameters.iter().map(|p| p.internal_name.clone()).collect_vec();

    return match &decoration.iter().map(|a| a.as_ref()).collect_vec()[..] {
        [
            Positioned { position: p1, value: ast::Term::Identifier(i)},
            Positioned { position: p2, value: ast::Term::Struct(args)}
        ] => {
            if i != "pattern" {
                return Err(RuntimeError::new("Unrecognized decoration.".to_string()))
            }

            let [a, b] = &args[..] else {
                return Err(RuntimeError::new("pattern decoration needs two arguments.".to_string()))
            };

            if a.key != ParameterKey::Positional || a.type_declaration.is_some() ||
                b.key != ParameterKey::Positional || b.type_declaration.is_some() {
                return Err(RuntimeError::new("pattern decoration arguments are faulty.".to_string()))
            }

            let precedence_group = match &b.value.iter().map(|p| p.as_ref()).collect_vec()[..] {
                [Positioned { position, value: ast::Term::Identifier(precedence) }] =>
                    scope.resolve_precedence_group(&precedence)?,
                _ => return Err(RuntimeError::new("Second argument to pattern needs to be a precedence name.".to_string()))
            };

            let parts: Vec<Box<PatternPart>> = a.value.iter()
                .map(|pterm| {
                    match &pterm.value {
                        ast::Term::Identifier(i) => {
                            Ok(Box::new(parameters.iter()
                                .position(|p| p == i)
                                .map(|p| PatternPart::Parameter(p))
                                .unwrap_or(PatternPart::Keyword(i.clone()))))
                        },
                        _ => Err(RuntimeError::new("Bad pattern.".to_string())),
                    }
                })
                .try_collect()?;

            Ok(Rc::new(Pattern {
                id: Uuid::new_v4(),
                precedence_group,
                parts,
                head: function,
            }))
        },
        _ => Err(RuntimeError::new("Unrecognized decoration.".to_string()))
    }
}
