use crate::program::types::{TypeProto, TypeUnit};
use crate::transpiler::python::{ast, FunctionContext};

pub fn transpile(type_def: &TypeProto, context: &FunctionContext) -> Box<ast::Expression> {
    match &type_def.unit {
        TypeUnit::Struct(s) => {
            let representation = &context.representations.type_ids.get(type_def).unwrap_or_else(|| panic!("Unable to find representation for type {:?}", s));
            Box::new(ast::Expression::NamedReference(context.names[representation].clone()))
        },
        TypeUnit::Generic(id) => panic!("Failed to transpile {:?}, generics shouldn't exist anymore at this point.", type_def),
        TypeUnit::Void => todo!(),
    }
}
