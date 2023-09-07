use crate::program::types::{TypeProto, TypeUnit};
use crate::transpiler::python::FunctionContext;

pub fn transpile(type_def: &TypeProto, context: &FunctionContext) -> String {
    match &type_def.unit {
        TypeUnit::Struct(s) => context.names[&context.struct_ids[type_def]].clone(),
        TypeUnit::Generic(_) => todo!(),
        TypeUnit::Any(id) => todo!("Failed to transpile Any<{}>", id),
        TypeUnit::MetaType => todo!(),
        TypeUnit::Void => todo!(),
        TypeUnit::Function(_) => todo!(),
    }
}
