use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;
use uuid::Uuid;
use crate::program::builtins::Builtins;
use crate::program::global::{BuiltinFunctionHint, FunctionImplementation};
use crate::program::types::TypeProto;

pub struct ClassContext<'a> {
    pub names: &'a HashMap<Uuid, String>,
    pub functions_by_id: &'a HashMap<Uuid, Rc<FunctionImplementation>>,
    pub builtins: &'a Builtins,
    pub builtin_hints: &'a HashMap<Uuid, BuiltinFunctionHint>,
    pub struct_ids: &'a HashMap<Box<TypeProto>, Uuid>,
}

pub fn transpile_class(stream: &mut (dyn Write), type_def: &TypeProto, context: &ClassContext) -> Result<(), std::io::Error> {
    // TODO If the type has no variables, we can fold it away from the program entirely
    let struct_id = context.struct_ids[type_def];
    write!(stream, "\n\nclass {}:\n    pass", context.names[&struct_id])?;
    Ok(())
}
