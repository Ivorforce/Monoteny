use std::io::Write;
use crate::transpiler::python::FunctionContext;
use crate::program::global::FunctionImplementation;
use crate::program::types::{TypeProto, TypeUnit};

pub fn dump(stream: &mut (dyn Write), implementation: &FunctionImplementation, context: &FunctionContext) -> Result<(), std::io::Error> {
    write!(stream, ":\n    \"\"\"\n    <Docstring TODO!>\n")?;

    if !implementation.head.interface.parameters.is_empty() {
        write!(stream, "\n    Args:\n")?;

        for (idx, parameter) in implementation.parameter_variables.iter().enumerate() {
            write!(stream, "        {}: ", context.names.get(&parameter.id).unwrap())?;
            transpile_type(stream, &parameter.type_, context)?;
            write!(stream, "\n")?;
        }
    }

    if !implementation.head.interface.return_type.unit.is_void() {
        write!(stream, "\n    Returns: ")?;
        transpile_type(stream, &implementation.head.interface.return_type, context)?;
        write!(stream, "\n")?;
    }

    write!(stream, "    \"\"\"\n")?;

    Ok(())
}

pub fn transpile_type(stream: &mut (dyn Write), type_def: &TypeProto, context: &FunctionContext) -> Result<(), std::io::Error> {
    match &type_def.unit {
        TypeUnit::Struct(s) => write!(stream, "{}", &context.names[&context.struct_ids[type_def]])?,
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
