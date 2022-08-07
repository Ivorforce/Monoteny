use std::io::Write;
use crate::transpiler::python::{get_external_name, TranspilerContext, types};
use crate::program::builtins::TenLangBuiltins;
use crate::linker::computation_tree::*;
use crate::program::types::Type;

pub fn dump(stream: &mut (dyn Write), function: &Function, context: &TranspilerContext) -> Result<(), std::io::Error> {
    write!(stream, ":\n    \"\"\"\n    <Docstring TODO!>\n")?;

    if !function.interface.parameters.is_empty() {
        write!(stream, "\n    Args:\n")?;

        for parameter in function.interface.parameters.iter() {
            write!(stream, "        {}: ", get_external_name(&parameter))?;
            transpile_type(stream, &parameter.variable.type_declaration, context)?;
            write!(stream, "\n")?;
        }
    }

    if let Some(return_type) = &function.interface.return_type {
        write!(stream, "\n    Returns: ")?;
        transpile_type(stream, return_type, context)?;
        write!(stream, "\n")?;
    }

    write!(stream, "    \"\"\"\n")?;

    Ok(())
}

pub fn transpile_type(stream: &mut (dyn Write), type_def: &Type, context: &TranspilerContext) -> Result<(), std::io::Error> {
    match type_def {
        Type::Primitive(n) => types::transpile_primitive(stream, n)?,
        Type::Struct(s) => types::transpile_struct(stream, s, context)?,
        Type::Monad(unit) => {
            transpile_type(stream, unit, context)?;
            write!(stream, "[?]")?;
        }
        Type::Function(_) => todo!(),
        Type::Generic(_) => todo!(),
        Type::MetaType(_) => todo!(),
        Type::PrecedenceGroup(_) => todo!(),
    }

    Ok(())
}
