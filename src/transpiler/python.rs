pub mod docstrings;
pub mod types;
pub mod builtins;

use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;
use guard::guard;
use itertools::zip_eq;
use uuid::Uuid;

use crate::program::builtins::TenLangBuiltins;
use crate::program::computation_tree::*;
use crate::program::functions::{FunctionPointer, FunctionPointerTarget, HumanFunctionInterface};
use crate::program::primitives;
use crate::program::traits::TraitConformanceDeclaration;
use crate::program::types::{ParameterKey, Type, TypeUnit, Variable};
use crate::transpiler::namespaces;


pub struct TranspilerContext<'a> {
    names: HashMap<Uuid, String>,
    functions_by_id: HashMap<Uuid, Rc<FunctionImplementation>>,
    builtins: &'a TenLangBuiltins,
}

pub fn transpile_program(stream: &mut (dyn Write), program: &Program, builtins: &TenLangBuiltins) -> Result<(), std::io::Error> {
    writeln!(stream, "import tenlang as tl")?;
    writeln!(stream, "import numpy as np")?;
    writeln!(stream, "import operator as op")?;
    writeln!(stream, "from numpy import int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, bool")?;
    writeln!(stream, "from typing import Any, Callable")?;

    let mut global_namespace = builtins::create(builtins);
    let mut file_namespace = global_namespace.add_sublevel();
    let mut object_namespace = namespaces::Level::new();
    let mut functions_by_id = HashMap::new();

    for trait_ in program.traits.iter() {
        todo!("Register names like the builtins do")
    }

    for function in program.functions.iter() {
        file_namespace.register_definition(function.implementation_id, &function.human_interface.alphanumeric_name);

        let function_namespace = file_namespace.add_sublevel();
        for (variable, name) in function.variable_names.iter() {
            function_namespace.register_definition(variable.id.clone(), name);
        }

        for declaration in function.conformance_delegations.values() {
            // The declaration will be a parameter later
            // TODO If two traits of the same type are registered, names will clash. We need to add aliases (later?)
            add_ids_as_name(declaration, &declaration.trait_.name, function_namespace);
        }

        functions_by_id.insert(function.implementation_id, Rc::clone(function));
    }

    let mut names = global_namespace.map_names();
    names.extend(object_namespace.map_names());

    let context = TranspilerContext {
        names,
        functions_by_id,
        builtins,
    };

    for function in program.functions.iter() {
        transpile_function(stream, function.as_ref(), &context)?
    }

    return Ok(())
}

fn add_ids_as_name(declaration: &Rc<TraitConformanceDeclaration>, name: &String, namespace: &mut namespaces::Level) {
    namespace.insert_keyword(declaration.id, name);

    for declaration in declaration.trait_requirements_conformance.values() {
        add_ids_as_name(declaration, name, namespace);
    }
}

fn add_injections_to_namespace(declaration: &Rc<TraitConformanceDeclaration>, namespace: &mut namespaces::Level) {
    for injected_function in declaration.function_implementations.values() {
        namespace.register_definition(injected_function.pointer_id.clone(), &injected_function.human_interface.alphanumeric_name);
    }

    for declaration in declaration.trait_requirements_conformance.values() {
        add_injections_to_namespace(declaration, namespace);
    }
}

pub fn transpile_function(stream: &mut (dyn Write), function: &FunctionImplementation, context: &TranspilerContext) -> Result<(), std::io::Error> {
    write!(stream, "\n\ndef {}(", context.names[&function.implementation_id])?;

    for (idx, (key, variable)) in function.human_interface.parameter_names.iter().enumerate() {
        write!(stream, "{}: ", get_external_name(key, idx))?;
        types::transpile(stream, &variable.type_declaration, context)?;
        write!(stream, ", ")?;
    }

    for declaration in function.conformance_delegations.values() {
        write!(stream, "{}: {}, ", context.names.get(&declaration.id).unwrap(), context.names.get(&declaration.trait_.id).unwrap())?;
    }

    write!(stream, ")")?;

    if let Some(return_type) = &function.machine_interface.return_type {
        write!(stream, " -> ", )?;
        types::transpile(stream, return_type, context)?;
    }

    docstrings::dump(stream, function, context)?;

    if function.statements.is_empty() {
        // No need to do conversions or anything else if we don't have a body.
        write!(stream, "    pass\n")?;
        return Ok(())
    }

    for (idx, (key, variable)) in function.human_interface.parameter_names.iter().enumerate() {
        let variable_name = context.names.get(&variable.id).unwrap();
        // TODO Rather than re-using the external name, we should map it to a new variable and register its name
        let external_name = get_external_name(key, idx);

        match &variable.type_declaration.unit {
            TypeUnit::Monad => {
                let unit = &variable.type_declaration.arguments[0].as_ref().unit;

                if let TypeUnit::Struct(s) = unit {
                    write!(
                        stream, "    {} = np.asarray({}, dtype=",
                        variable_name,
                        external_name
                    )?;
                    types::transpile_struct(stream, s, context)?;
                    write!(stream, ")\n")?;
                }
                else if let TypeUnit::Primitive(primitive) = unit {
                    write!(
                        stream, "    {} = np.asarray({}, dtype=",
                        variable_name,
                        external_name
                    )?;
                    types::transpile_primitive(stream, primitive)?;
                    write!(stream, ")\n")?;
                }
                else {
                    panic!("Can't have a nested monad in numpy.")
                }
            }
            _ => {
                if variable_name == &external_name {
                    continue
                }

                writeln!(
                    stream, "    {} = {}", variable_name, external_name,
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
                let variable_name = context.names.get(&variable.id).unwrap();

                write!(stream, "    {} = ", variable_name)?;
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
            let variable_name = context.names.get(&variable.id).unwrap();

            write!(stream, "{}", variable_name)?;
        }
        ExpressionOperation::FunctionCall { function, arguments, binding } => {
            if try_transpile_binary_operator(stream, function, arguments, context)? {
                // no-op
            }
            else if try_transpile_unary_operator(stream, function, arguments, context)? {
                // no-op
            }
            else {
                match &function.target {
                    // Can reference the static function
                    FunctionPointerTarget::Static { .. } => write!(stream, "{}", context.names[&function.pointer_id])?,
                    // Have to reference the function by trait
                    FunctionPointerTarget::Polymorphic { declaration_id, abstract_function } => {
                        write!(stream, "{}.{}", &context.names[declaration_id], context.names[&abstract_function.pointer_id])?;
                    }
                }
                write!(stream, "(")?;

                let mut arguments_left = arguments.len() + function.machine_interface.requirements.len();

                for (param_key, variable) in function.human_interface.parameter_names.iter() {
                    let argument = arguments.get(variable).unwrap();

                    if let ParameterKey::Name(name) = &param_key {
                        write!(stream, "{}=", name)?;
                    }
                    // Otherwise, pass as *args

                    transpile_expression(stream, &argument, context)?;

                    arguments_left -= 1;
                    if arguments_left > 0 {
                        write!(stream, ", ")?;
                    }
                }

                for requirement in function.machine_interface.requirements.iter() {
                    let declaration = &context.functions_by_id[&function.pointer_id].conformance_delegations[requirement];

                    let param_name = context.names.get(&declaration.id).unwrap();
                    let arg_name = context.names.get(&binding.resolution[requirement].id).unwrap();
                    write!(stream, "{}={}", param_name, arg_name)?;

                    arguments_left -= 1;
                    if arguments_left > 0 {
                        write!(stream, ", ")?;
                    }
                }

                write!(stream, ")")?;
            }
        },
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
            for (idx, (args, function)) in zip_eq(arguments.windows(2), functions.iter()).enumerate() {
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

pub fn try_transpile_unary_operator(stream: &mut (dyn Write), function: &Rc<FunctionPointer>, arguments: &HashMap<Rc<Variable>, Box<Expression>>, context: &TranspilerContext) -> Result<bool, std::io::Error> {
    if arguments.len() != 1 {
        return Ok(false);
    }

    let expression = arguments.values().next().unwrap();

    // TODO We can probably avoid unnecessary parentheses here and in the other operators if we ask the expression for its (python) precedence, and compare it with ours.
    let mut transpile_unary_operator = |name: &str| -> Result<bool, std::io::Error> {
        write!(stream, "{}", name)?;
        transpile_maybe_parenthesized_expression(stream, expression.as_ref(), context)?;
        Ok(true)
    };

    if context.builtins.operators.positive.contains(function) {
        return transpile_unary_operator("+");
    }
    else if context.builtins.operators.negative.contains(function) {
        return transpile_unary_operator("-");
    }
    else if function.as_ref() == context.builtins.operators.not.as_ref() {
        return transpile_unary_operator("not ");
    }

    return Ok(false);
}

pub fn try_transpile_binary_operator(stream: &mut (dyn Write), function: &Rc<FunctionPointer>, arguments: &HashMap<Rc<Variable>, Box<Expression>>, context: &TranspilerContext) -> Result<bool, std::io::Error> {
    let arguments: Vec<&Box<Expression>> = function.human_interface.parameter_names.iter()
        .map(|(_, var)| arguments.get(var).unwrap())
        .collect();
    if arguments.len() != 2 {
        return Ok(false);
    }

    let lhs = arguments[0];
    let rhs = arguments[1];

    let mut transpile_binary_operator = |name: &str| -> Result<bool, std::io::Error> {
        transpile_maybe_parenthesized_expression(stream, &lhs, context)?;
        write!(stream, " {} ", name)?;
        transpile_maybe_parenthesized_expression(stream, &rhs, context)?;

        Ok(true)
    };

    // TODO And and Or exist but work only for boolean arguments, not tensors.
    //  We could make use of them if the arguments are booleans and the result is too.
    if function.as_ref() == context.builtins.operators.and.as_ref() {
        return transpile_binary_operator("&");
    }
    else if function.as_ref() == context.builtins.operators.or.as_ref() {
        return transpile_binary_operator("|");
    }

    else if context.builtins.operators.equal_to.contains(function) {
        return transpile_binary_operator("==");
    }
    else if context.builtins.operators.not_equal_to.contains(function) {
        return transpile_binary_operator("!=");
    }

    else if context.builtins.operators.greater_than.contains(function) {
        return transpile_binary_operator(">");
    }
    else if context.builtins.operators.greater_than_or_equal_to.contains(function) {
        return transpile_binary_operator(">=");
    }
    else if context.builtins.operators.lesser_than.contains(function) {
        return transpile_binary_operator("<");
    }
    else if context.builtins.operators.lesser_than_or_equal_to.contains(function) {
        return transpile_binary_operator("<=");
    }

    else if context.builtins.operators.add.contains(function) {
        return transpile_binary_operator("+");
    }
    else if context.builtins.operators.subtract.contains(function) {
        return transpile_binary_operator("-");
    }
    else if context.builtins.operators.multiply.contains(function) {
        return transpile_binary_operator("*");
    }
    else if context.builtins.operators.divide.contains(function) {
        return transpile_binary_operator("/");
    }
    else if context.builtins.operators.exponentiate.contains(function) {
        return transpile_binary_operator("**");
    }
    else if context.builtins.operators.modulo.contains(function) {
        return transpile_binary_operator("%");
    }

    return Ok(false);
}

pub fn get_external_name(key: &ParameterKey, idx: usize) -> String {
    match &key {
        ParameterKey::Name(name) => name.clone(),
        // Int keying is not supported in python. Let's prefix via underscore.
        ParameterKey::Int(n) => String::from(format!("_{}", n)),
        // None keying is not supported; let's use two underscores as prefix! lol
        ParameterKey::Positional => String::from(format!("__{}", idx))
    }
}

pub fn is_simple(operation: &ExpressionOperation) -> bool {
    match operation {
        ExpressionOperation::Primitive(_) => true,
        ExpressionOperation::VariableLookup(_) => true,
        ExpressionOperation::StringLiteral(_) => true,
        ExpressionOperation::ArrayLiteral(_) => true,
        ExpressionOperation::FunctionCall { .. } => false,
        ExpressionOperation::MemberLookup(_, _) => false,
        ExpressionOperation::PairwiseOperations { .. } => false,
    }
}
