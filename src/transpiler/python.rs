pub mod docstrings;
pub mod types;
pub mod builtins;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::ops::DerefMut;
use std::rc::Rc;
use guard::guard;
use itertools::{Itertools, zip_eq};
use uuid::Uuid;
use regex;
use crate::interpreter;

use crate::program::builtins::Builtins;
use crate::program::computation_tree::*;
use crate::program::functions::{FunctionPointer, FunctionCallType, FunctionInterface, ParameterKey};
use crate::program::{primitives, Program};
use crate::program::allocation::Reference;
use crate::program::generics::TypeForest;
use crate::program::global::{FunctionImplementation};
use crate::program::traits::{TraitConformanceDeclaration, TraitConformanceRequirement};
use crate::program::types::{TypeProto, TypeUnit};
use crate::transpiler::namespaces;
use crate::transpiler::python::docstrings::transpile_type;

pub struct TranspilerContext<'a> {
    names: &'a HashMap<Uuid, String>,
    functions_by_id: &'a HashMap<Uuid, Rc<FunctionImplementation>>,
    builtins: &'a Builtins,
    expressions: &'a ExpressionForest,
    types: &'a TypeForest,
}

pub fn transpile_program(stream: &mut (dyn Write), program: &Program, builtins: Rc<Builtins>) -> Result<(), std::io::Error> {
    writeln!(stream, "import numpy as np")?;
    writeln!(stream, "import math")?;
    writeln!(stream, "import operator as op")?;
    writeln!(stream, "from numpy import int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, bool")?;
    writeln!(stream, "from typing import Any, Callable")?;

    let mut global_namespace = builtins::create(&builtins);
    let mut file_namespace = global_namespace.add_sublevel();
    let mut object_namespace = namespaces::Level::new();
    let mut functions_by_id = HashMap::new();

    for trait_ in program.module.traits.iter() {
        todo!("Register names like the builtins do")
    }

    for implementation in program.function_implementations.values() {
        file_namespace.register_definition(implementation.implementation_id, &implementation.pointer.name);

        let function_namespace = file_namespace.add_sublevel();
        for (variable, name) in implementation.variable_names.iter() {
            function_namespace.register_definition(variable.id.clone(), name);
        }

        for declaration in implementation.conformance_delegations.values() {
            // The declaration will be a parameter later
            // TODO If two traits of the same type are registered, names will clash. We need to add aliases (later?)
            add_ids_as_name(declaration, &declaration.trait_.name, function_namespace);
        }

        functions_by_id.insert(implementation.implementation_id, Rc::clone(implementation));
    }

    let mut names = global_namespace.map_names();
    names.extend(object_namespace.map_names());
    let names = Rc::new(names);

    let exported_blocks: Rc<RefCell<HashMap<Uuid, String>>> = Rc::new(RefCell::new(HashMap::new()));
    let internal_blocks: Rc<RefCell<HashMap<Uuid, String>>> = Rc::new(RefCell::new(HashMap::new()));

    interpreter::run::transpile(program, &Rc::clone(&builtins), &|implementation| {
        let context = TranspilerContext {
            names: &names,
            functions_by_id: &functions_by_id,
            builtins: &builtins,
            expressions: &implementation.expression_forest,
            types: &implementation.type_forest
        };

        let mut block = Vec::new();
        transpile_function(&mut block, implementation, &context).unwrap();
        exported_blocks.borrow_mut().deref_mut().insert(implementation.implementation_id, String::from_utf8(block).unwrap());
        // todo!("We need to unfold and transpile the functions that 'implementation' uses.");
    });

    for block in exported_blocks.borrow_mut().deref_mut().values() {
        write!(stream, "{}\n\n", block)?;
    }

    writeln!(stream, "# ========================== ======== ============================")?;
    writeln!(stream, "# ========================== Internal ============================")?;
    writeln!(stream, "# ========================== ======== ============================")?;

    for block in internal_blocks.borrow_mut().deref_mut().values() {
        write!(stream, "{}\n\n", block)?;
    }

    writeln!(stream, "\n__all__ = [")?;
    for name in exported_blocks.borrow_mut().deref_mut().keys().map(|x| &names[x]).sorted() {
        writeln!(stream, "    \"{}\",", name)?;
    }
    writeln!(stream, "]")?;

    if let Some(main_function) = program.find_annotated("main") {
        write!(stream, "\n\nif __name__ == '__main__':\n    {}()\n", names.get(&main_function.implementation_id).unwrap())?;
    }

    return Ok(())
}

fn add_ids_as_name(declaration: &Rc<TraitConformanceDeclaration>, name: &String, namespace: &mut namespaces::Level) {
    namespace.insert_keyword(declaration.id, name);

    for declaration in declaration.trait_binding.resolution.values() {
        add_ids_as_name(declaration, name, namespace);
    }
}

fn add_injections_to_namespace(declaration: &Rc<TraitConformanceDeclaration>, namespace: &mut namespaces::Level) {
    for injected_function in declaration.function_binding.values() {
        namespace.register_definition(injected_function.pointer_id.clone(), &injected_function.name);
    }

    for declaration in declaration.trait_binding.resolution.values() {
        add_injections_to_namespace(declaration, namespace);
    }
}

pub fn transpile_function(stream: &mut (dyn Write), function: &FunctionImplementation, context: &TranspilerContext) -> Result<(), std::io::Error> {
    write!(stream, "\n\ndef {}(", context.names[&function.implementation_id])?;

    for (idx, parameter) in function.pointer.target.interface.parameters.iter().enumerate() {
        write!(stream, "{}: ", context.names.get(&parameter.target.id).unwrap())?;
        types::transpile(stream, &parameter.target.type_, context)?;
        write!(stream, ", ")?;
    }

    // TODO Only required when conformance isn't fully resolved
    // for declaration in function.conformance_delegations.values() {
    //     write!(stream, "{}: {}, ", context.names.get(&declaration.id).unwrap(), context.names.get(&declaration.trait_.id).unwrap())?;
    // }

    write!(stream, ")")?;

    if !function.pointer.target.interface.return_type.unit.is_void() {
        write!(stream, " -> ", )?;
        types::transpile(stream, &function.pointer.target.interface.return_type, context)?;
    }

    docstrings::dump(stream, function, context)?;

    if function.statements.is_empty() {
        // No need to do conversions or anything else if we don't have a body.
        write!(stream, "    pass\n")?;
        return Ok(())
    }

    for (idx, parameter) in function.pointer.target.interface.parameters.iter().enumerate() {
        let variable_name = context.names.get(&parameter.target.id).unwrap();
        let external_name = variable_name;  // external names are not supported in python

        match &parameter.target.type_.unit {
            TypeUnit::Monad => {
                let unit = &parameter.target.type_.arguments[0].as_ref().unit;

                if let TypeUnit::Struct(s) = unit {
                    write!(
                        stream, "    {} = np.asarray({}, dtype=",
                        variable_name,
                        external_name
                    )?;
                    types::transpile_struct(stream, s, context)?;
                    write!(stream, ")\n")?;
                }
                else {
                    panic!("Can't have a nested monad in numpy.")
                }
            }
            _ => {}
        }
    }

    for statement in function.statements.iter() {
        match statement.as_ref() {
            Statement::Return(Some(expression)) => {
                write!(stream, "    return ")?;
                transpile_expression(stream, expression.clone(), context)?;
            }
            Statement::Return(None) => {
                write!(stream, "    return")?;
            }
            Statement::VariableAssignment(variable, expression) => {
                let variable_name = context.names.get(&variable.id).unwrap();

                write!(stream, "    {} = ", variable_name)?;
                transpile_expression(stream, expression.clone(), context)?;
            }
            Statement::Expression(expression) => {
                write!(stream, "    ")?;
                transpile_expression(stream, expression.clone(), context)?;
            }
        }

        write!(stream, "\n")?;
    }

    Ok(())
}

pub fn transpile_expression(stream: &mut (dyn Write), expression: ExpressionID, context: &TranspilerContext) -> Result<(), std::io::Error> {
    match &context.expressions.operations.get(&expression).unwrap() {
        ExpressionOperation::StringLiteral(string) => {
            write!(stream, "\"{}\"", escape_string(&string))?;
        }
        ExpressionOperation::VariableLookup(variable) => {
            let variable_name = context.names.get(&variable.id).unwrap();

            write!(stream, "{}", variable_name)?;
        }
        ExpressionOperation::FunctionCall { function, binding } => {
            let arguments = context.expressions.arguments.get(&expression).unwrap();

            if
                try_transpile_keyword(stream, function, context)?
                || try_transpile_binary_operator(stream, function, arguments, context)?
                || try_transpile_constant(stream, function, arguments, &expression, context)?
                || try_transpile_unary_operator(stream, function, arguments, context)?
                || try_transpile_literal(stream, function, arguments, &expression, context)?
            {
                // no-op
            }
            else {
                match &function.call_type {
                    // Can reference the static function
                    FunctionCallType::Static => {
                        guard!(let Some(name) = context.names.get(&function.pointer_id) else {
                            panic!("Couldn't find name in python: {:?}", function)
                        });
                        write!(stream, "{}", name)?
                    },
                    // Have to reference the function by trait
                    FunctionCallType::Polymorphic { requirement, abstract_function } => {
                        write!(stream, "{}.{}", &context.names[todo!("We used to look for 'declaration ID', but that was weird, where is the name stored?")], context.names[&function.pointer_id])?;
                    }
                }
                write!(stream, "(")?;

                // TODO Only required when we're forward-passing unresolved requirements
                let requirements: [&Rc<TraitConformanceRequirement>; 0] = [];  // function.target.interface.requirements
                let mut arguments_left = arguments.len() + requirements.len();

                for (parameter, argument) in zip_eq(function.target.interface.parameters.iter(), arguments.iter()) {
                    if let ParameterKey::Name(name) = &parameter.external_key {
                        write!(stream, "{}=", name)?;
                    }
                    // Otherwise, pass as *args

                    transpile_expression(stream, argument.clone(), context)?;

                    arguments_left -= 1;
                    if arguments_left > 0 {
                        write!(stream, ", ")?;
                    }
                }

                for requirement in requirements {
                    let implementation = &context.functions_by_id[&function.pointer_id];
                    let declaration = &implementation.conformance_delegations[requirement];

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
        ExpressionOperation::ArrayLiteral => {
            todo!()
            // write!(stream, "[")?;
            // for (idx, expression) in expressions.iter().enumerate() {
            //     transpile_expression(stream, expression, context)?;
            //
            //     if idx < expressions.len() -1 {
            //         write!(stream, ", ")?;
            //     }
            // }
            // write!(stream, "]")?;
        },
        ExpressionOperation::PairwiseOperations { functions } => {
            todo!()
            // // TODO Unfortunately, python's a > b > c syntax does not support non-bool results.
            // //  For true boolean results, we could actually use it for readability.
            // // This is suboptimal, but easy: Just compute arguments twice lol.
            // for (idx, (args, function)) in zip_eq(arguments.windows(2), functions.iter()).enumerate() {
            //     // TODO Use try_transpile_binary_operator / try_transpile_unary_operator so we correctly map names / alphanumeric names.
            //     write!(stream, "(")?;
            //     transpile_expression(stream, &args[0], context)?;
            //     write!(stream, ") {} (", function.name)?;
            //     transpile_expression(stream, &args[1], context)?;
            //     write!(stream, ")")?;
            //
            //     if idx < functions.len() - 1 {
            //         write!(stream, " and ")?;
            //     }
            // }
        }
    }

    Ok(())
}

pub fn transpile_maybe_parenthesized_expression(stream: &mut (dyn Write), expression: ExpressionID, context: &TranspilerContext) -> Result<(), std::io::Error> {
    if is_simple(&context.expressions.operations.get(&expression).unwrap()) {
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

pub fn try_transpile_unary_operator(stream: &mut (dyn Write), function: &Rc<FunctionPointer>, arguments: &Vec<ExpressionID>, context: &TranspilerContext) -> Result<bool, std::io::Error> {
    guard!(let [expression] = arguments[..] else {
        return Ok(false)
    });

    // TODO We can probably avoid unnecessary parentheses here and in the other operators if we ask the expression for its (python) precedence, and compare it with ours.
    for (collection, operator) in [
        (&context.builtins.core.primitive_fns.positive, "+"),
        (&context.builtins.core.primitive_fns.negative, "-"),

        // TODO This is not ideal
        (&HashMap::from([(primitives::Type::Bool, Rc::clone(&context.builtins.core.primitive_fns.not))]), "not "),
    ] {
        // TODO values().contains is not ideal
        if !(collection.values().contains(function)) {
            continue;
        }

        write!(stream, "{}", operator)?;
        transpile_maybe_parenthesized_expression(stream, expression.clone(), context)?;

        return Ok(true);
    }

    return Ok(false);
}

pub fn try_transpile_binary_operator(stream: &mut (dyn Write), function: &Rc<FunctionPointer>, arguments: &Vec<ExpressionID>, context: &TranspilerContext) -> Result<bool, std::io::Error> {
    guard!(let [lhs, rhs] = arguments[..] else {
        return Ok(false)
    });

    for (collection, operator) in [
        // TODO This is not ideal
        (&HashMap::from([(primitives::Type::Bool, Rc::clone(&context.builtins.core.primitive_fns.and))]), "and"),
        (&HashMap::from([(primitives::Type::Bool, Rc::clone(&context.builtins.core.primitive_fns.or))]), "or"),

        (&context.builtins.core.primitive_fns.equal_to, "=="),
        (&context.builtins.core.primitive_fns.not_equal_to, "!="),

        (&context.builtins.core.primitive_fns.greater_than, ">"),
        (&context.builtins.core.primitive_fns.greater_than_or_equal_to, ">="),
        (&context.builtins.core.primitive_fns.lesser_than, "<"),
        (&context.builtins.core.primitive_fns.lesser_than_or_equal_to, "<="),

        (&context.builtins.core.primitive_fns.add, "+"),
        (&context.builtins.core.primitive_fns.subtract, "-"),
        (&context.builtins.core.primitive_fns.multiply, "*"),
        (&context.builtins.core.primitive_fns.divide, "/"),

        (&context.builtins.core.primitive_fns.exponent, "**"),
        (&context.builtins.core.primitive_fns.modulo, "%"),
    ] {
        // TODO values().contains is not ideal
        if !(collection.values().contains(function)) {
            continue;
        }

        transpile_maybe_parenthesized_expression(stream, lhs.clone(), context)?;
        write!(stream, " {} ", operator)?;
        transpile_maybe_parenthesized_expression(stream, rhs.clone(), context)?;

        return Ok(true);
    }

    return Ok(false);
}

pub fn try_transpile_literal(stream: &mut (dyn Write), function: &Rc<FunctionPointer>, arguments: &Vec<ExpressionID>, expression_id: &ExpressionID, context: &TranspilerContext) -> Result<bool, std::io::Error> {
    guard!(let [argument] = &arguments[..] else {
        return Ok(false)
    });

    guard!(let ExpressionOperation::StringLiteral(literal) = &context.expressions.operations[argument] else {
        return Ok(false)
    });

    let is_float = regex::Regex::new("^[0-9]+\\.[0-9]*$").unwrap();
    let is_int = regex::Regex::new("^[0-9]+$").unwrap();

    // TODO values().contains is not ideal
    if context.builtins.core.primitive_fns.parse_int_literal.values().contains(function) && is_int.is_match(literal) {
        types::transpile(stream, &context.types.resolve_binding_alias(expression_id).unwrap(), context)?;
        write!(stream, "({})",  literal)?;
        return Ok(true);
    }
    else if context.builtins.core.primitive_fns.parse_float_literal.values().contains(function) && is_float.is_match(literal) {
        types::transpile(stream, &context.types.resolve_binding_alias(expression_id).unwrap(), context)?;
        write!(stream, "({})", literal)?;
        return Ok(true);
    }

    Ok(false)
}

pub fn try_transpile_keyword(stream: &mut (dyn Write), function: &Rc<FunctionPointer>, context: &TranspilerContext) -> Result<bool, std::io::Error> {
    if function == &context.builtins.common.true_ {
        write!(stream, "True")?;
        return Ok(true)
    }

    if function == &context.builtins.common.false_ {
        write!(stream, "False")?;
        return Ok(true)
    }

    Ok(false)
}

pub fn try_transpile_constant(stream: &mut (dyn Write), function: &Rc<FunctionPointer>, arguments: &Vec<ExpressionID>, expression_id: &ExpressionID, context: &TranspilerContext) -> Result<bool, std::io::Error> {
    if !arguments.is_empty() {
        return Ok(false);
    };

    if !match &context.types.resolve_binding_alias(expression_id).unwrap().unit {
        TypeUnit::Struct(s) => {
            // TODO values().contains is not ideal
            context.builtins.core.primitives.values().contains(s)
        },
        _ => false,
    } {
        return Ok(false)
    }

    if function == &context.builtins.math.pi {
        transpile_type(stream, &context.types.resolve_binding_alias(expression_id).unwrap(), context)?;
        write!(stream, "(np.pi)")?;
        return Ok(true)
    }
    else if function == &context.builtins.math.tau {
        transpile_type(stream, &context.types.resolve_binding_alias(expression_id).unwrap(), context)?;
        write!(stream, "(np.pi * 2)")?;
        return Ok(true)
    }
    else if function == &context.builtins.math.e {
        transpile_type(stream, &context.types.resolve_binding_alias(expression_id).unwrap(), context)?;
        write!(stream, "(np.e)")?;
        return Ok(true)
    }

    Ok(false)
}

pub fn is_simple(operation: &ExpressionOperation) -> bool {
    match operation {
        ExpressionOperation::VariableLookup(_) => true,
        ExpressionOperation::StringLiteral(_) => true,
        ExpressionOperation::ArrayLiteral => true,
        ExpressionOperation::FunctionCall { .. } => false,
        ExpressionOperation::PairwiseOperations { .. } => false,
    }
}
