use std::collections::HashMap;
use uuid::Uuid;
use crate::interpreter::Runtime;
use crate::program::types::TypeProto;
use crate::transpiler::python::ast;
use crate::transpiler::python::representations::Representations;

pub struct ClassContext<'a> {
    pub names: &'a HashMap<Uuid, String>,
    pub representations: &'a Representations,

    pub runtime: &'a Runtime,
}

pub fn transpile_class(type_def: &TypeProto, context: &ClassContext) -> Box<ast::Class> {
    // TODO If the type has no variables, we can fold it away from the program entirely
    let struct_id = context.representations.type_ids[type_def];
    Box::new(ast::Class {
        name: context.names[&struct_id].clone(),
    })
}
