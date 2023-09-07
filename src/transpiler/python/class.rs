use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;
use crate::program::builtins::Builtins;
use crate::program::global::{BuiltinFunctionHint, FunctionImplementation};
use crate::program::types::TypeProto;
use crate::transpiler::python::syntax;

pub struct ClassContext<'a> {
    pub names: &'a HashMap<Uuid, String>,
    pub functions_by_id: &'a HashMap<Uuid, Rc<FunctionImplementation>>,
    pub builtins: &'a Builtins,
    pub builtin_hints: &'a HashMap<Uuid, BuiltinFunctionHint>,
    pub struct_ids: &'a HashMap<Box<TypeProto>, Uuid>,
}

pub fn transpile_class(type_def: &TypeProto, context: &ClassContext) -> Box<syntax::Class> {
    // TODO If the type has no variables, we can fold it away from the program entirely
    let struct_id = context.struct_ids[type_def];
    Box::new(syntax::Class {
        name: context.names[&struct_id].clone(),
    })
}
