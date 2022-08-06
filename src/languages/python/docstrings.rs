use std::io::Write;
use crate::languages::python::{get_external_name, types};
use crate::program::builtins::TenLangBuiltins;
use crate::linker::computation_tree::*;
use crate::program::types::Type;

pub fn dump(stream: &mut (dyn Write), function: &Function, builtins: &TenLangBuiltins) -> Result<(), std::io::Error> {
    write!(stream, ":\n    \"\"\"\n    <Docstring TODO!>\n")?;

    if !function.interface.parameters.is_empty() {
        write!(stream, "\n    Args:\n")?;

        for parameter in function.interface.parameters.iter() {
            write!(stream, "        {}: ", get_external_name(&parameter))?;
            transpile_type(stream, &parameter.variable.type_declaration, builtins)?;
            write!(stream, "\n")?;
        }
    }

    if let Some(return_type) = &function.interface.return_type {
        write!(stream, "\n    Returns: ")?;
        transpile_type(stream, return_type, builtins)?;
        write!(stream, "\n")?;
    }

    write!(stream, "    \"\"\"\n")?;

    Ok(())
}

pub fn transpile_type(stream: &mut (dyn Write), type_def: &Type, builtins: &TenLangBuiltins) -> Result<(), std::io::Error> {
    match type_def {
        Type::Primitive(n) => types::transpile_primitive(stream, n)?,
        Type::Struct(s) => types::transpile_struct(stream, s, builtins)?,
        Type::NDArray(atom) => {
            transpile_type(stream, atom, builtins)?;
            write!(stream, "[?]")?;
        },
        Type::Function(_) => todo!(),
        Type::Generic(_) => todo!(),
        Type::MetaType(_) => todo!()
    }

    Ok(())
}
