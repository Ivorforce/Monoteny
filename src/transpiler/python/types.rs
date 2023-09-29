use crate::program::types::{TypeProto, TypeUnit};
use crate::transpiler::python::FunctionContext;

pub fn transpile(type_def: &TypeProto, context: &FunctionContext) -> String {
    match &type_def.unit {
        TypeUnit::Struct(s) => context.names[&context.representations.type_ids[type_def]].clone(),
        TypeUnit::Generic(id) => panic!("Failed to transpile {:?}, generics shouldn't exist anymore at this point.", type_def),
        TypeUnit::Any(id) => todo!("Failed to transpile {:?}; generic transpilation is not supported yet.", type_def),
        TypeUnit::MetaType => todo!(),
        TypeUnit::Void => todo!(),
        TypeUnit::Function(_) => todo!(),
    }
}
