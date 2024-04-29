use std::rc::Rc;

use itertools::Itertools;
use strum::IntoEnumIterator;
use uuid::Uuid;

use crate::ast;
use crate::error::{RResult, RuntimeError, TryCollectMany};
use crate::parser::expressions;
use crate::parser::grammar::{OperatorAssociativity, PrecedenceGroup};
use crate::program::functions::ParameterKey;
use crate::resolver::scopes;

pub fn resolve_precedence_order(call_struct: &ast::Struct, scope: &scopes::Scope) -> RResult<Vec<Rc<PrecedenceGroup>>> {
    let order: Vec<Rc<PrecedenceGroup>> = call_struct.arguments.iter().map(|arg| {
        let ParameterKey::Name(name) = &arg.value.key else {
            return Err(RuntimeError::error("Not a named argument.").in_range(arg.position.clone()).to_array())
        };
        if arg.value.type_declaration.is_some() {
            return Err(RuntimeError::error("Unexpected type declaration.").in_range(arg.position.clone()).to_array())
        }

        let associativity = resolve_associativity(&arg.value.value, scope)?;

        Ok(Rc::new(PrecedenceGroup {
            trait_id: Uuid::new_v4(),
            name: name.to_string(),
            associativity,
        }))
    }).try_collect_many()?;

    order.iter().filter(|x| x.associativity == OperatorAssociativity::LeftUnary).at_most_one()
        .map_err(|_| RuntimeError::error("Cannot declare two LeftUnary associativities.").to_array())?;
    order.iter().filter(|x| x.associativity == OperatorAssociativity::RightUnary).at_most_one()
        .map_err(|_| RuntimeError::error("Cannot declare two RightUnary associativities.").to_array())?;

    Ok(order)
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

