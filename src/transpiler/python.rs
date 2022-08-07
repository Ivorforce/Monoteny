pub mod docstrings;
pub mod types;
pub mod builtins;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::io::Write;
use std::iter::zip;
use std::rc::Rc;
use guard::guard;
use uuid::Uuid;

use crate::program::builtins::TenLangBuiltins;
use crate::linker::computation_tree::*;
use crate::program::primitives;
use crate::program::types::{FunctionForm, FunctionInterface, NamedParameter, ParameterKey, Type};
use crate::transpiler::namespaces;


pub struct TranspilerContext<'a> {
    names: HashMap<Uuid, String>,
    builtins: &'a TenLangBuiltins,
}

pub fn transpile_program(stream: &mut (dyn Write), program: &Program, builtins: &TenLangBuiltins) -> Result<(), std::io::Error> {
    writeln!(stream, "import numpy as np")?;
    writeln!(stream, "from numpy import int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, bool")?;

    let mut builtin_namespaces = builtins::create(builtins);
    let mut namespaces = builtin_namespaces.add_sublevel();

    for function in program.functions.iter() {
        namespaces.register_definition(function.interface.id, &function.interface.alphanumeric_name);
        register_names(&function.statements, namespaces.add_sublevel(), builtins);
    }

    let context = TranspilerContext {
        names: builtin_namespaces.map_names(),
        builtins
    };

    for function in program.functions.iter() {
        transpile_function(stream, function.as_ref(), &context)?
    }

    return Ok(())
}

pub fn register_names(statements: &Vec<Box<Statement>>, namespace: &mut namespaces::Level, builtins: &TenLangBuiltins) {
    for statement in statements {
        match statement.as_ref() {
            Statement::VariableAssignment(variable, _) => {
                namespace.register_definition(variable.id.clone(), &variable.name);
            }
            _ => {}
        }
    }
}

pub fn transpile_function(stream: &mut (dyn Write), function: &Function, context: &TranspilerContext) -> Result<(), std::io::Error> {
    write!(stream, "\n\ndef {}(", context.names[&function.interface.id])?;

    // TODO Can we somehow transpile function.interface.is_member_function?
    for parameter in function.interface.parameters.iter() {
        write!(stream, "{}: ", get_external_name(&parameter))?;
        types::transpile(stream, &parameter.variable.type_declaration, context)?;
        write!(stream, ",")?;
    }

    write!(stream, ")")?;

    if let Some(return_type) = &function.interface.return_type {
        write!(stream, " -> ", )?;
        types::transpile(stream, return_type, context)?;
    }

    docstrings::dump(stream, function, context)?;

    if function.statements.is_empty() {
        // No need to do conversions or anything else if we don't have a body.
        write!(stream, "    pass\n")?;
        return Ok(())
    }

    for parameter in function.interface.parameters.iter() {
        match parameter.variable.type_declaration.borrow() {
            Type::Monad(unit) => {
                if let Type::Struct(s) = unit.as_ref() {
                    write!(
                        stream, "    {} = np.asarray({}, dtype=",
                        parameter.variable.name,
                        get_external_name(parameter)
                    )?;
                    types::transpile_struct(stream, s, context)?;
                    write!(stream, ")\n")?;
                }
                else if let Type::Primitive(primitive) = unit.as_ref() {
                    write!(
                        stream, "    {} = np.asarray({}, dtype=",
                        parameter.variable.name,
                        get_external_name(parameter)
                    )?;
                    types::transpile_primitive(stream, primitive)?;
                    write!(stream, ")\n")?;
                }
                else {
                    panic!("Can't have a nested monad in numpy.")
                }
            }
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
                transpile_expression(stream, &expression, context)?;
            }
            Statement::Return(None) => {
                write!(stream, "    return")?;
            }
            Statement::VariableAssignment(variable, expression) => {
                write!(stream, "    {} = ", variable.name)?;
                transpile_expression(stream, expression, context)?;
            }
            Statement::Expression(expression) => {
                write!(stream, "    ")?;
                transpile_expression(stream, expression, context)?;
            }
        }

        write!(stream, "\n")?;
    }

    Ok(())
}

pub fn transpile_expression(stream: &mut (dyn Write), expression: &Expression, context: &TranspilerContext) -> Result<(), std::io::Error> {
    match &expression.operation.as_ref() {
        ExpressionOperation::Primitive(value) => {
            types::transpile_primitive_value(stream, value)?;
        }
        ExpressionOperation::StringLiteral(string) => {
            write!(stream, "\"{}\"", escape_string(&string))?;
        }
        ExpressionOperation::VariableLookup(variable) => {
            write!(stream, "{}", variable.name)?;
        }
        ExpressionOperation::StaticFunctionCall { function, arguments } => {
            if try_transpile_binary_operator(stream, function, arguments, context)? {
                // no-op
            }
            else if try_transpile_unary_operator(stream, function, arguments, context)? {
                // no-op
            }
            else {
                write!(stream, "{}(", context.names[&function.id])?;
                for (idx, argument) in arguments.iter().enumerate() {
                    if let ParameterKey::Name(name) = &argument.key {
                        write!(stream, "{}=", name)?;
                    }
                    transpile_expression(stream, &argument.value, context)?;

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
                transpile_expression(stream, expression, context)?;

                if idx < expressions.len() -1 {
                    write!(stream, ", ")?;
                }
            }
            write!(stream, "]")?;
        },
        ExpressionOperation::PairwiseOperations { arguments, functions } => {
            // TODO Unfortunately, python's a > b > c syntax does not support non-bool results.
            //  For true boolean results, we could actually use it for readability.
            // This is suboptimal, but easy: Just compute arguments twice lol.
            for (idx, (args, function)) in zip(arguments.windows(2), functions.iter()).enumerate() {
                // TODO Use try_transpile_binary_operator / try_transpile_unary_operator so we correctly map names / alphanumeric names.
                write!(stream, "(")?;
                transpile_expression(stream, &args[0], context)?;
                write!(stream, ") {} (", function.name)?;
                transpile_expression(stream, &args[1], context)?;
                write!(stream, ")")?;

                if idx < functions.len() - 1 {
                    write!(stream, " and ")?;
                }
            }
        }
    }

    Ok(())
}

pub fn transpile_maybe_parenthesized_expression(stream: &mut (dyn Write), expression: &Expression, context: &TranspilerContext) -> Result<(), std::io::Error> {
    if is_simple(&expression.operation) {
        transpile_expression(stream, expression, context)?;
    }
    else {
        write!(stream, "(")?;
        transpile_expression(stream, expression, context)?;
        write!(stream, ")")?;
    }

    Ok(())
}

pub fn escape_string(string: &String) -> String {
    let string = string.replace("\\", "\\\\");
    let string = string.replace("\"", "\\\"");
    return string
}

pub fn try_transpile_unary_operator(stream: &mut (dyn Write), interface: &Rc<FunctionInterface>, arguments: &Vec<Box<PassedArgument>>, context: &TranspilerContext) -> Result<bool, std::io::Error> {
    guard!(let [expression] = &arguments[..] else {
        return Ok(false);
    });

    // TODO We can probably avoid unnecessary parentheses here and in the other operators if we ask the expression for its (python) precedence, and compare it with ours.
    let mut transpile_unary_operator = |name: &str| -> Result<bool, std::io::Error> {
        write!(stream, "{}", name)?;
        transpile_maybe_parenthesized_expression(stream, &expression.value, context)?;
        Ok(true)
    };

    if context.builtins.operators.positive.contains(interface) {
        return transpile_unary_operator("+");
    }
    else if context.builtins.operators.negative.contains(interface) {
        return transpile_unary_operator("-");
    }
    else if interface.as_ref() == context.builtins.operators.not.as_ref() {
        return transpile_unary_operator("not ");
    }

    return Ok(false);
}

pub fn try_transpile_binary_operator(stream: &mut (dyn Write), interface: &Rc<FunctionInterface>, arguments: &Vec<Box<PassedArgument>>, context: &TranspilerContext) -> Result<bool, std::io::Error> {
    guard!(let [lhs, rhs] = &arguments[..] else {
        return Ok(false);
    });

    let mut transpile_binary_operator = |name: &str| -> Result<bool, std::io::Error> {
        transpile_maybe_parenthesized_expression(stream, &lhs.value, context)?;
        write!(stream, " {} ", name)?;
        transpile_maybe_parenthesized_expression(stream, &rhs.value, context)?;

        Ok(true)
    };

    // TODO And and Or exist but work only for boolean arguments, not tensors.
    //  We could make use of them if the arguments are booleans and the result is too.
    if interface.as_ref() == context.builtins.operators.and.as_ref() {
        return transpile_binary_operator("&");
    }
    else if interface.as_ref() == context.builtins.operators.or.as_ref() {
        return transpile_binary_operator("|");
    }

    else if context.builtins.operators.equal_to.contains(interface) {
        return transpile_binary_operator("==");
    }
    else if context.builtins.operators.not_equal_to.contains(interface) {
        return transpile_binary_operator("!=");
    }

    else if context.builtins.operators.greater_than.contains(interface) {
        return transpile_binary_operator(">");
    }
    else if context.builtins.operators.greater_than_or_equal_to.contains(interface) {
        return transpile_binary_operator(">=");
    }
    else if context.builtins.operators.lesser_than.contains(interface) {
        return transpile_binary_operator("<");
    }
    else if context.builtins.operators.lesser_than_or_equal_to.contains(interface) {
        return transpile_binary_operator("<=");
    }

    else if context.builtins.operators.add.contains(interface) {
        return transpile_binary_operator("+");
    }
    else if context.builtins.operators.subtract.contains(interface) {
        return transpile_binary_operator("-");
    }
    else if context.builtins.operators.multiply.contains(interface) {
        return transpile_binary_operator("*");
    }
    else if context.builtins.operators.divide.contains(interface) {
        return transpile_binary_operator("/");
    }
    else if context.builtins.operators.exponentiate.contains(interface) {
        return transpile_binary_operator("**");
    }
    else if context.builtins.operators.modulo.contains(interface) {
        return transpile_binary_operator("%");
    }

    return Ok(false);
}

pub fn get_external_name(parameter: &NamedParameter) -> String {
    match &parameter.external_key {
        ParameterKey::Name(key) => key.clone(),
        // Int keying is not supported in python. Just use the variable name.
        ParameterKey::Int(_) => parameter.variable.name.clone(),
    }
}

pub fn is_simple(operation: &ExpressionOperation) -> bool {
    match operation {
        ExpressionOperation::Primitive(_) => true,
        ExpressionOperation::VariableLookup(_) => true,
        ExpressionOperation::StringLiteral(_) => true,
        ExpressionOperation::ArrayLiteral(_) => true,
        ExpressionOperation::StaticFunctionCall { .. } => false,
        ExpressionOperation::MemberLookup(_, _) => false,
        ExpressionOperation::PairwiseOperations { .. } => false,
    }
}
