use std::io::Write;
use crate::transpiler::python::{get_external_name, TranspilerContext, types};
use crate::program::builtins::TenLangBuiltins;
use crate::linker::computation_tree::*;
use crate::program::types::{Type, TypeUnit};

pub fn dump(stream: &mut (dyn Write), function: &FunctionImplementation, context: &TranspilerContext) -> Result<(), std::io::Error> {
    write!(stream, ":\n    \"\"\"\n    <Docstring TODO!>\n")?;

    if !function.interface.parameter_names.is_empty() {
        write!(stream, "\n    Args:\n")?;

        for (idx, (key, variable)) in function.interface.parameter_names.iter().enumerate() {
            write!(stream, "        {}: ", get_external_name(key, idx))?;
            transpile_type(stream, &variable.type_declaration, context)?;
            write!(stream, "\n")?;
        }
    }

    if let Some(return_type) = &function.interface.machine_interface.return_type {
        write!(stream, "\n    Returns: ")?;
        transpile_type(stream, return_type, context)?;
        write!(stream, "\n")?;
    }

    write!(stream, "    \"\"\"\n")?;

    Ok(())
}

pub fn transpile_type(stream: &mut (dyn Write), type_def: &Type, context: &TranspilerContext) -> Result<(), std::io::Error> {
    match &type_def.unit {
        TypeUnit::Primitive(n) => types::transpile_primitive(stream, n)?,
        TypeUnit::Struct(s) => types::transpile_struct(stream, s, context)?,
        TypeUnit::Trait(t) => todo!(),
        TypeUnit::Monad => {
            transpile_type(stream, &type_def.arguments[0], context)?;
            write!(stream, "[?]")?;
        }
        TypeUnit::Function(_) => todo!(),
        TypeUnit::Generic(_) => todo!(),
        TypeUnit::Any(_) => todo!(),
        TypeUnit::MetaType => todo!(),
        TypeUnit::PrecedenceGroup(_) => todo!(),
    }

    Ok(())
}
