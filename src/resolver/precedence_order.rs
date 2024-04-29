use std::rc::Rc;

use itertools::Itertools;
use strum::IntoEnumIterator;
use uuid::Uuid;

use crate::ast;
use crate::error::{ErrInRange, RResult, RuntimeError, TryCollectMany};
use crate::parser::expressions;
use crate::parser::grammar::{OperatorAssociativity, PrecedenceGroup};
use crate::resolver::{interpreter_mock, scopes};

pub fn resolve_precedence_order(body: &ast::Expression, scope: &scopes::Scope) -> RResult<Vec<Rc<PrecedenceGroup>>> {
    let error = RuntimeError::error("precedence_order! needs an array literal body.").to_array();

    let parsed = expressions::parse(body, &scope.grammar)?;

    let expressions::Value::ArrayLiteral(array) = &parsed.value else {
        return Err(error);
    };

    let order: Vec<Rc<PrecedenceGroup>> = array.arguments.iter().map(|arg| {
        if arg.value.key.is_some() || arg.value.type_declaration.is_some() {
            return Err(error.clone())
        }

        resolve_precedence_group(&arg.value.value, scope)
    }).try_collect_many().err_in_range(&parsed.position)?;

    order.iter().filter(|x| x.associativity == OperatorAssociativity::LeftUnary).at_most_one()
        .map_err(|_| RuntimeError::error("Cannot declare two LeftUnary associativities.").to_array())?;
    order.iter().filter(|x| x.associativity == OperatorAssociativity::RightUnary).at_most_one()
        .map_err(|_| RuntimeError::error("Cannot declare two RightUnary associativities.").to_array())?;

    Ok(order)
}

pub fn resolve_precedence_group(body: &ast::Expression, scope: &scopes::Scope) -> RResult<Rc<PrecedenceGroup>> {
    let error = RuntimeError::error("Precedence group needs form name(associativity).").to_array();

    let parsed = expressions::parse(body, &scope.grammar)?;

    let expressions::Value::FunctionCall(target, call_struct) = &parsed.value else {
        return Err(error);
    };

    let expressions::Value::Identifier(name) = &target.value else {
        return Err(error);
    };

    let body = interpreter_mock::plain_parameter("Precedence Group", call_struct)?;
    let associativity = resolve_associativity(body, scope).err_in_range(&parsed.position)?;

    Ok(Rc::new(PrecedenceGroup {
        trait_id: Uuid::new_v4(),
        name: name.to_string(),
        associativity,
    }))
}

pub fn resolve_associativity(body: &ast::Expression, scope: &scopes::Scope) -> RResult<OperatorAssociativity> {
    let error = RuntimeError::error(
        format!("Operator associativity needs to be one of {:?}.", OperatorAssociativity::iter().collect_vec()).as_str()
    ).to_array();

    let parsed = expressions::parse(body, &scope.grammar)?;

    let expressions::Value::Identifier(identifier) = &parsed.value else {
        return Err(error);
    };

    let Ok(associativity) = OperatorAssociativity::iter().filter(|a| &a.to_string() == identifier.as_str()).exactly_one() else {
        return Err(error)
    };

    Ok(associativity)
}

