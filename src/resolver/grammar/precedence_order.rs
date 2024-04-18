use std::rc::Rc;

use itertools::Itertools;
use strum::IntoEnumIterator;
use uuid::Uuid;

use crate::error::{ErrInRange, RResult, RuntimeError};
use crate::resolver::grammar::{OperatorAssociativity, PrecedenceGroup};
use crate::resolver::interpreter_mock;
use crate::parser::ast;
use crate::parser::ast::Term;

pub fn resolve_precedence_order(body: &ast::Expression) -> RResult<Vec<Rc<PrecedenceGroup>>> {
    let error = RuntimeError::new(format!("@precedence_order needs an array literal body."));

    let order: Vec<Rc<PrecedenceGroup>> = match &body[..] {
        [term] => {
            match &term.value {
                Term::Array(arguments) => {
                    arguments.iter().map(|arg| {
                        if arg.key.is_some() || arg.type_declaration.is_some() {
                            return Err(error.clone())
                        }

                        resolve_precedence_group(&arg.value)
                    }).try_collect().err_in_range(&term.position)?
                }
                _ => return Err(error),
            }
        }
        _ => return Err(error),
    };

    order.iter().filter(|x| x.associativity == OperatorAssociativity::LeftUnary).at_most_one()
        .map_err(|_| RuntimeError::new("Cannot declare two LeftUnary associativities.".to_string()))?;
    order.iter().filter(|x| x.associativity == OperatorAssociativity::RightUnary).at_most_one()
        .map_err(|_| RuntimeError::new("Cannot declare two RightUnary associativities.".to_string()))?;

    Ok(order)
}

pub fn resolve_precedence_group(body: &ast::Expression) -> RResult<Rc<PrecedenceGroup>> {
    let error = RuntimeError::new(format!("Precedence group needs form name(associativity)."));

    match &body[..] {
        [l, r] => {
            match (&l.value, &r.value) {
                (Term::Identifier(name), Term::Struct(struct_args)) => {
                    let body = interpreter_mock::plain_parameter("Precedence Group", struct_args)?;
                    let associativity = resolve_associativity(body).err_in_range(&r.position)?;

                    Ok(Rc::new(PrecedenceGroup {
                        trait_id: Uuid::new_v4(),
                        name: name.to_string(),
                        associativity,
                    }))
                }
                _ => Err(error).err_in_range(&l.position)
            }
        }
        _ => Err(error),
    }
}

pub fn resolve_associativity(body: &ast::Expression) -> RResult<OperatorAssociativity> {
    let error = RuntimeError::new(format!("Operator associativity needs to be one of {:?}.", OperatorAssociativity::iter().collect_vec()));
    match &body[..] {
        [arg] => {
            let Term::Identifier(name) = &arg.value else {
                return Err(error)
            };

            let associativity = OperatorAssociativity::iter().filter(|a| &a.to_string() == name).exactly_one();
            let Ok(associativity) = associativity else {
                return Err(error)
            };

            Ok(associativity)
        }
        _ => Err(error)
    }
}

