use std::io::Write;
use crate::transpiler::python::{TranspilerContext, types};
use crate::program::builtins::Builtins;
use crate::program::computation_tree::*;
use crate::program::global::FunctionImplementation;
use crate::program::types::{TypeProto, TypeUnit};

pub fn dump(stream: &mut (dyn Write), implementation: &FunctionImplementation, context: &TranspilerContext) -> Result<(), std::io::Error> {
    write!(stream, ":\n    \"\"\"\n    <Docstring TODO!>\n")?;

    if !implementation.pointer.target.interface.parameters.is_empty() {
        write!(stream, "\n    Args:\n")?;

        for (idx, parameter) in implementation.pointer.target.interface.parameters.iter().enumerate() {
            write!(stream, "        {}: ", context.names.get(&parameter.target.id).unwrap())?;
            transpile_type(stream, &parameter.target.type_, context)?;
            write!(stream, "\n")?;
        }
    }

    if !implementation.pointer.target.interface.return_type.unit.is_void() {
        write!(stream, "\n    Returns: ")?;
        transpile_type(stream, &implementation.pointer.target.interface.return_type, context)?;
        write!(stream, "\n")?;
    }

    write!(stream, "    \"\"\"\n")?;

    Ok(())
}

pub fn transpile_type(stream: &mut (dyn Write), type_def: &TypeProto, context: &TranspilerContext) -> Result<(), std::io::Error> {
    match &type_def.unit {
        TypeUnit::Struct(s) => types::transpile_struct(stream, s, context)?,
        TypeUnit::Monad => {
            transpile_type(stream, &type_def.arguments[0], context)?;
            write!(stream, "[?]")?;
        }
        TypeUnit::Generic(_) => todo!(),
        TypeUnit::Any(_) => write!(stream, "Any")?,  // TODO Use generics instead
        TypeUnit::MetaType => todo!(),
        TypeUnit::Void => todo!(),
        TypeUnit::Function(_) => todo!(),
    }

    Ok(())
}
