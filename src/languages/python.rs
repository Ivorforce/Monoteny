use std::borrow::Borrow;
use std::io::Write;
use guard::guard;
use crate::linker::builtins::TenLangBuiltins;
use crate::linker::computation_tree::*;

pub fn transpile_program(stream: &mut (dyn Write), program: &Program) -> Result<(), std::io::Error> {
    writeln!(stream, "import numpy as np")?;

    for function in program.functions.iter() {
        transpile_function(stream, function.as_ref(), &program.builtins)?
    }

    return Ok(())
}

pub fn transpile_function(stream: &mut (dyn Write), function: &Function, builtins: &TenLangBuiltins) -> Result<(), std::io::Error> {
    write!(stream, "\n\ndef {}(", function.interface.name)?;

    for parameter in function.interface.parameters.iter() {
        write!(stream, "{}: ", get_external_name(&parameter))?;
        transpile_type(stream, &parameter.variable.type_declaration)?;
        write!(stream, ",")?;
    }

    write!(stream, ")")?;

    if let Some(return_type) = &function.interface.return_type {
        write!(stream, " -> ", )?;
        transpile_type(stream, return_type)?;
    }

    write!(stream, ":\n    \"\"\"\n    <Docstring TODO!>\n")?;

    if !function.interface.parameters.is_empty() {
        write!(stream, "\n    Args:\n")?;

        for parameter in function.interface.parameters.iter() {
            write!(stream, "        {}: ", get_external_name(&parameter))?;
            transpile_type_for_docstring(stream, &parameter.variable.type_declaration)?;
            write!(stream, "\n")?;
        }
    }

    if let Some(return_type) = &function.interface.return_type {
        write!(stream, "\n    Returns: ")?;
        transpile_type_for_docstring(stream, return_type)?;
        write!(stream, "\n")?;
    }

    write!(stream, "    \"\"\"\n")?;

    if function.statements.is_empty() {
        // No need to do conversions or anything else if we don't have a body.
        write!(stream, "    pass\n")?;
        return Ok(())
    }

    for parameter in function.interface.parameters.iter() {
        match parameter.variable.type_declaration.borrow() {
            Type::NDArray(atom) => {
                if let Type::Identifier(atom) = atom.as_ref() {
                    write!(
                        stream, "    {} = np.asarray({}, dtype=",
                        parameter.variable.name,
                        get_external_name(parameter)
                    )?;
                    transpile_atom_type(stream, atom)?;
                    write!(stream, ")\n")?;
                }
                else if let Type::Primitive(primitive) = atom.as_ref() {
                    write!(
                        stream, "    {} = np.asarray({}, dtype=",
                        parameter.variable.name,
                        get_external_name(parameter)
                    )?;
                    transpile_primitive_type(stream, primitive)?;
                    write!(stream, ")\n")?;
                }
                else {
                    panic!("Can't have a non-atom ndarray in numpy.")
                }
            },
            _ => {
                let external_name = get_external_name(&parameter);

                if parameter.variable.name == external_name {
                    continue
                }

                writeln!(
                    stream, "    {} = {}", parameter.variable.name, external_name,
                )?;
            }
        }
    }

    for statement in function.statements.iter() {
        match statement.as_ref() {
            Statement::Return(Some(expression)) => {
                write!(stream, "    return ")?;
                transpile_expression(stream, &expression, builtins)?;
            }
            Statement::Return(None) => {
                write!(stream, "    return")?;
            }
            Statement::VariableAssignment(variable, expression) => {
                write!(stream, "    {} = ", variable.name)?;
                transpile_expression(stream, expression, builtins)?;
            }
            Statement::Expression(expression) => {
                write!(stream, "    ")?;
                transpile_expression(stream, expression, builtins)?;
            }
        }

        write!(stream, "\n")?;
    }

    Ok(())
}

pub fn transpile_primitive_type(stream: &mut (dyn Write), type_def: &PrimitiveType) -> Result<(), std::io::Error> {
    use PrimitiveType::*;
    match type_def {
        Bool => write!(stream, "np.bool")?,
        Int8 => write!(stream, "np.int8")?,
        Int16 => write!(stream, "np.int16")?,
        Int32 => write!(stream, "np.int32")?,
        Int64 => write!(stream, "np.int64")?,
        Int128 => write!(stream, "np.int128")?,
        UInt8 => write!(stream, "np.uint8")?,
        UInt16 => write!(stream, "np.uint16")?,
        UInt32 => write!(stream, "np.uint32")?,
        UInt64 => write!(stream, "np.uint64")?,
        UInt128 => write!(stream, "np.uint128")?,
        Float32 => write!(stream, "np.float32")?,
        Float64 => write!(stream, "np.float64")?,
    }

    Ok(())
}

pub fn transpile_atom_type(stream: &mut (dyn Write), type_def: &String) -> Result<(), std::io::Error> {
    write!(stream, "{}", match type_def.as_str() {
        "String" => "str",
        atom => atom
    })?;

    Ok(())
}


pub fn transpile_type(stream: &mut (dyn Write), type_def: &Type) -> Result<(), std::io::Error> {
    match type_def.borrow() {
        Type::Primitive(n) => transpile_primitive_type(stream, n)?,
        Type::Identifier(atom) => transpile_atom_type(stream, &atom)?,
        Type::NDArray(_) => write!(stream, "np.ndarray")?,
        Type::Function(_) => todo!(),
        Type::Generic(_) => todo!(),
    }

    Ok(())
}

pub fn transpile_type_for_docstring(stream: &mut (dyn Write), type_def: &Type) -> Result<(), std::io::Error> {
    match type_def.borrow() {
        Type::Primitive(n) => transpile_primitive_type(stream, n)?,
        Type::Identifier(atom) => transpile_atom_type(stream, atom)?,
        Type::NDArray(atom) => {
            transpile_type_for_docstring(stream, atom)?;
            write!(stream, "[?]")?;
        },
        Type::Function(_) => todo!(),
        Type::Generic(_) => todo!(),
    }

    Ok(())
}

pub fn transpile_expression(stream: &mut (dyn Write), expression: &Expression, builtins: &TenLangBuiltins) -> Result<(), std::io::Error> {
    match &expression.operation.as_ref() {
        ExpressionOperation::Primitive(literal) => {
            match &literal {
                Primitive::Bool(n) => write!(stream, "{}", (if *n { "True" } else { "False" }))?,
                Primitive::Int8(n) => write!(stream, "np.int8({})", n)?,
                Primitive::Int16(n) => write!(stream, "np.int16({})", n)?,
                Primitive::Int32(n) => write!(stream, "np.int32({})", n)?,
                Primitive::Int64(n) => write!(stream, "np.int64({})", n)?,
                Primitive::Int128(n) => write!(stream, "np.int128({})", n)?,
                Primitive::UInt8(n) => write!(stream, "np.uint8({})", n)?,
                Primitive::UInt16(n) => write!(stream, "np.uint16({})", n)?,
                Primitive::UInt32(n) => write!(stream, "np.uint32({})", n)?,
                Primitive::UInt64(n) => write!(stream, "np.uint64({})", n)?,
                Primitive::UInt128(n) => write!(stream, "np.uint128({})", n)?,
                Primitive::Float32(n) => write!(stream, "np.float32({})", n)?,
                Primitive::Float64(n) => write!(stream, "np.float64({})", n)?,
            }
        }
        ExpressionOperation::StringLiteral(string) => {
            write!(stream, "\"{}\"", escape_string(&string))?;
        }
        ExpressionOperation::VariableLookup(variable) => {
            write!(stream, "{}", variable.name)?;
        }
        ExpressionOperation::StaticFunctionCall { function, arguments } => {
            if try_transpile_binary_operator(stream, function, arguments, builtins)? {
                // no-op
            }
            else {
                // TODO We should make sure it calls the correct function even when shadowed.
                write!(stream, "{}(", function.name)?;
                for (idx, argument) in arguments.iter().enumerate() {
                    if let ParameterKey::Name(name) = &argument.key {
                        write!(stream, "{}=", name)?;
                    }
                    transpile_expression(stream, &argument.value, builtins)?;

                    if idx < arguments.len() -1 {
                        write!(stream, ", ")?;
                    }
                }
                write!(stream, ")")?;
            }
        }
        ExpressionOperation::MemberLookup(_, _) => todo!(),
        ExpressionOperation::ArrayLiteral(expressions) => {
            write!(stream, "[")?;
            for (idx, expression) in expressions.iter().enumerate() {
                transpile_expression(stream, expression, builtins)?;

                if idx < expressions.len() -1 {
                    write!(stream, ", ")?;
                }
            }
            write!(stream, "]")?;
        },
    }

    Ok(())
}

pub fn escape_string(string: &String) -> String {
    let string = string.replace("\\", "\\\\");
    let string = string.replace("\"", "\\\"");
    return string
}

pub fn try_transpile_unary_operator(stream: &mut (dyn Write), interface: &FunctionInterface, arguments: &Vec<Box<PassedArgument>>, builtins: &TenLangBuiltins) -> Result<bool, std::io::Error> {
    guard!(let [expression] = &arguments[..] else {
        return Ok(false);
    });

    // TODO We can probably avoid unnecessary parentheses here and in the other operators if we ask the expression for its (python) precedence, and compare it with ours.
    let mut transpile_unary_operator = |name: &str| -> Result<bool, std::io::Error> {
        write!(stream, "{}(", name)?;
        transpile_expression(stream, &expression.value, builtins)?;
        write!(stream, ")")?;
        Ok(true)
    };

    if interface == builtins.operators.positive.as_ref() {
        return transpile_unary_operator("+");
    }
    else if interface == builtins.operators.negative.as_ref() {
        return transpile_unary_operator("-");
    }
    else if interface == builtins.operators.not.as_ref() {
        return transpile_unary_operator("not ");
    }

    return Ok(false);
}

pub fn try_transpile_binary_operator(stream: &mut (dyn Write), interface: &FunctionInterface, arguments: &Vec<Box<PassedArgument>>, builtins: &TenLangBuiltins) -> Result<bool, std::io::Error> {
    guard!(let [lhs, rhs] = &arguments[..] else {
        return Ok(false);
    });

    let mut transpile_binary_operator = |name: &str| -> Result<bool, std::io::Error> {
        write!(stream, "(")?;
        transpile_expression(stream, &lhs.value, builtins)?;
        write!(stream, ") {} (", name)?;
        transpile_expression(stream, &rhs.value, builtins)?;
        write!(stream, ")")?;

        Ok(true)
    };

    // TODO And and Or exist but work only for boolean arguments, not tensors.
    //  We could make use of them if the arguments are booleans and the result is too.
    if interface == builtins.operators.and.as_ref() {
        return transpile_binary_operator("&");
    }
    else if interface == builtins.operators.or.as_ref() {
        return transpile_binary_operator("|");
    }

    else if interface == builtins.operators.equal_to.as_ref() {
        return transpile_binary_operator("==");
    }
    else if interface == builtins.operators.not_equal_to.as_ref() {
        return transpile_binary_operator("!=");
    }

    else if interface == builtins.operators.greater_than.as_ref() {
        return transpile_binary_operator(">");
    }
    else if interface == builtins.operators.greater_than_or_equal_to.as_ref() {
        return transpile_binary_operator(">=");
    }
    else if interface == builtins.operators.lesser_than.as_ref() {
        return transpile_binary_operator("<");
    }
    else if interface == builtins.operators.lesser_than_or_equal_to.as_ref() {
        return transpile_binary_operator("<=");
    }

    else if interface == builtins.operators.add.as_ref() {
        return transpile_binary_operator("+");
    }
    else if interface == builtins.operators.subtract.as_ref() {
        return transpile_binary_operator("-");
    }
    else if interface == builtins.operators.multiply.as_ref() {
        return transpile_binary_operator("*");
    }
    else if interface == builtins.operators.divide.as_ref() {
        return transpile_binary_operator("/");
    }
    else if interface == builtins.operators.to_the_power_of.as_ref() {
        return transpile_binary_operator("**");
    }
    else if interface == builtins.operators.modulo.as_ref() {
        return transpile_binary_operator("%");
    }

    return Ok(false);
}

pub fn get_external_name(parameter: &Parameter) -> String {
    match &parameter.external_key {
        ParameterKey::Name(key) => key.clone(),
        // Int keying is not supported in python. Just use the variable name.
        ParameterKey::Int(_) => parameter.variable.name.clone(),
    }
}
