use crate::program::types::{TypeProto, TypeUnit};
use crate::transpiler::python::{ast, FunctionContext};

pub fn transpile(type_def: &TypeProto, context: &FunctionContext) -> Box<ast::Expression> {
    match &type_def.unit {
        TypeUnit::Struct(s) => Box::new(ast::Expression::NamedReference(context.names[&context.representations.type_ids[type_def]].clone())),
        TypeUnit::Generic(id) => panic!("Failed to transpile {:?}, generics shouldn't exist anymore at this point.", type_def),
        TypeUnit::MetaType => todo!(),
        TypeUnit::Void => todo!(),
        TypeUnit::Function(_) => todo!(),
    }
}
