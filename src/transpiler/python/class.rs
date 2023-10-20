use std::collections::HashMap;
use uuid::Uuid;
use crate::program::types::{TypeProto, TypeUnit};
use crate::transpiler::python::ast;
use crate::transpiler::python::representations::Representations;

pub struct ClassContext<'a> {
    pub names: &'a HashMap<Uuid, String>,
    pub representations: &'a Representations,
}

pub fn transpile_class(type_def: &TypeProto, context: &ClassContext) -> Box<ast::Class> {
    // TODO If the type has no variables, we can fold it away from the program entirely
    let struct_id = context.representations.type_ids[type_def];
    let mut statements = vec![];

    // TODO Need to account for bindings
    match &type_def.unit {
        TypeUnit::Struct(struct_) => {
            for hint in &struct_.field_hints {
                statements.push(Box::new(ast::Statement::VariableAssignment {
                    target: Box::new(ast::Expression::NamedReference(hint.name.clone())),
                    value: None,
                    type_annotation: Some(Box::new(ast::Expression::NamedReference(context.names[&context.representations.type_ids[&hint.type_]].clone()))),
                }))
            }
        }
        _ => panic!()
    }

    Box::new(ast::Class {
        name: context.names[&struct_id].clone(),
        statements,
    })
}
