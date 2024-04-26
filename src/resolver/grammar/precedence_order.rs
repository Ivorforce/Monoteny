use std::rc::Rc;

use itertools::Itertools;
use strum::IntoEnumIterator;
use uuid::Uuid;

use crate::error::{ErrInRange, RResult, RuntimeError, TryCollectMany};
use crate::resolver::grammar::{OperatorAssociativity, PrecedenceGroup};
use crate::resolver::interpreter_mock;
use crate::parser::ast;
use crate::parser::ast::Term;

pub fn resolve_precedence_order(body: &ast::Expression) -> RResult<Vec<Rc<PrecedenceGroup>>> {
    let error = RuntimeError::error("@precedence_order needs an array literal body.").to_array();

    let [term] = &body[..] else {
        return Err(error);
    };

    let Term::Array(array) = &term.value else {
        return Err(error);
    };

    let order = array.arguments.iter().map(|arg| {
        if arg.value.key.is_some() || arg.value.type_declaration.is_some() {
            return Err(error.clone())
        }

        resolve_precedence_group(&arg.value.value)
    }).try_collect_many().err_in_range(&term.position)?;

    order.iter().filter(|x| x.associativity == OperatorAssociativity::LeftUnary).at_most_one()
        .map_err(|_| RuntimeError::error("Cannot declare two LeftUnary associativities.").to_array())?;
    order.iter().filter(|x| x.associativity == OperatorAssociativity::RightUnary).at_most_one()
        .map_err(|_| RuntimeError::error("Cannot declare two RightUnary associativities.").to_array())?;

    Ok(order)
}

pub fn resolve_precedence_group(body: &ast::Expression) -> RResult<Rc<PrecedenceGroup>> {
    let error = RuntimeError::error("Precedence group needs form name(associativity).").to_array();

    let [l, r] = &body[..] else {
        return Err(error);
    };

    let (Term::Identifier(name), Term::Struct(struct_args)) = (&l.value, &r.value) else {
        return Err(error).err_in_range(&l.position);
    };

    let body = interpreter_mock::plain_parameter("Precedence Group", struct_args)?;
    let associativity = resolve_associativity(body).err_in_range(&r.position)?;

    Ok(Rc::new(PrecedenceGroup {
        trait_id: Uuid::new_v4(),
        name: name.to_string(),
        associativity,
    }))
}

pub fn resolve_associativity(body: &ast::Expression) -> RResult<OperatorAssociativity> {
    let error = RuntimeError::error(
        format!("Operator associativity needs to be one of {:?}.", OperatorAssociativity::iter().collect_vec()).as_str()
    ).to_array();

    let [arg] = &body[..] else {
        return Err(error);
    };

    let Term::Identifier(name) = &arg.value else {
        return Err(error)
    };

    let Ok(associativity) = OperatorAssociativity::iter().filter(|a| &a.to_string() == name).exactly_one() else {
        return Err(error)
    };

    Ok(associativity)
}

