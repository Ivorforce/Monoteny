pub mod docstrings;
pub mod types;
pub mod builtins;

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::ops::DerefMut;
use std::rc::Rc;
use guard::guard;
use itertools::{Itertools, zip_eq};
use uuid::Uuid;
use regex;
use crate::generic_unfolding::FunctionUnfolder;
use crate::interpreter;

use crate::program::builtins::Builtins;
use crate::program::computation_tree::*;
use crate::program::functions::{FunctionPointer, FunctionCallType, ParameterKey};
use crate::program::{find_annotated, primitives, Program};
use crate::program::calls::FunctionBinding;
use crate::program::generics::TypeForest;
use crate::program::global::{FunctionImplementation};
use crate::program::traits::{RequirementsFulfillment, TraitBinding};
use crate::program::types::TypeUnit;
use crate::transpiler::namespaces;
use crate::transpiler::python::docstrings::transpile_type;

pub struct TranspilerContext<'a> {
    names: &'a HashMap<Uuid, String>,
    functions_by_id: &'a HashMap<Uuid, Rc<FunctionImplementation>>,
    builtins: &'a Builtins,
    expressions: &'a ExpressionForest,
    types: &'a TypeForest,
}

pub fn transpile_program(stream: &mut (dyn Write), program: &Program, builtins: &Rc<Builtins>) -> Result<(), std::io::Error> {
    let mut global_namespace = builtins::create(&builtins);
    let mut file_namespace = global_namespace.add_sublevel();
    let mut object_namespace = namespaces::Level::new();
    let mut functions_by_id = HashMap::new();

    for implementation in program.module.function_implementations.values()
        .chain(builtins.module_by_name.values().flat_map(|module| module.function_implementations.values())) {
        functions_by_id.insert(implementation.implementation_id, Rc::clone(implementation));
    }

    let exported_symbols: Rc<RefCell<Vec<Rc<FunctionImplementation>>>> = Rc::new(RefCell::new(vec![]));
    let unfolder: Rc<RefCell<FunctionUnfolder>> = Rc::new(RefCell::new(FunctionUnfolder::new()));

    // Run interpreter

    interpreter::run::transpile(program, &Rc::clone(&builtins), &|implementation| {
        let unfolded_function = unfolder.borrow_mut().deref_mut().unfold_anonymous(
            implementation,
            &Rc::new(FunctionBinding {
                // The implementation's pointer is fine.
                pointer: Rc::clone(&implementation.pointer),
                // The resolution SHOULD be empty: The function is transpiled WITH its generics.
                // Unless generics are bound in the transpile directive, which is TODO
                requirements_fulfillment: RequirementsFulfillment::empty(),
            }),
            &|f| functions_by_id.contains_key(&f.pointer.pointer_id)  // TODO If no, it's *probably* a builtin, but we should probably check for realsies
        );

        exported_symbols.borrow_mut().deref_mut().push(unfolded_function);
    });

    // Find and unfold internal symbols
    let mut exported_symbols_ = exported_symbols.borrow_mut();
    let exported_symbols = exported_symbols_.deref_mut();
    let mut unfolder_ = unfolder.borrow_mut();
    let unfolder = unfolder_.deref_mut();

    let mut internal_symbols: Vec<Rc<FunctionImplementation>> = vec![];
    while let Some(used_symbol) = unfolder.new_mappable_calls.pop() {
        // TODO Use underscore names?
        let replacement_symbol = Rc::clone(&unfolder.mapped_calls[&used_symbol]);
        let implementation = &functions_by_id[&used_symbol.pointer.pointer_id];

        let unfolded_implementation = unfolder.unfold_anonymous(
            implementation,
            &replacement_symbol,
            &|f| functions_by_id.contains_key(&f.pointer.pointer_id)   // TODO If no, it's *probably* a builtin, but we should probably check for realsies)
        );

        internal_symbols.push(unfolded_implementation);
    }

    // Register symbols

    for implementation in exported_symbols.iter() {
        // TODO Register with priority over internal symbols
        file_namespace.register_definition(implementation.pointer.pointer_id, &implementation.pointer.name);
    }

    for implementation in internal_symbols.iter() {
        file_namespace.register_definition(implementation.pointer.pointer_id, &implementation.pointer.name);
    }

    for implementation in exported_symbols.iter().chain(internal_symbols.iter()) {
        let function_namespace = file_namespace.add_sublevel();
        for (variable, name) in implementation.variable_names.iter() {
            function_namespace.register_definition(variable.id.clone(), name);
        }
    }

    let mut names = global_namespace.map_names();
    names.extend(object_namespace.map_names());

    // Write to stream

    writeln!(stream, "import numpy as np")?;
    writeln!(stream, "import math")?;
    writeln!(stream, "import operator as op")?;
    writeln!(stream, "from numpy import int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, bool")?;
    writeln!(stream, "from typing import Any, Callable")?;

    for implementation in exported_symbols.iter() {
        let context = TranspilerContext {
            names: &names,
            functions_by_id: &functions_by_id,
            builtins: &builtins,
            expressions: &implementation.expression_forest,
            types: &implementation.type_forest
        };

        transpile_function(stream, implementation, &context).unwrap();
    }

    write!(stream, "\n\n")?;
    writeln!(stream, "# ========================== ======== ============================")?;
    writeln!(stream, "# ========================== Internal ============================")?;
    writeln!(stream, "# ========================== ======== ============================")?;

    for implementation in internal_symbols.iter() {
        let context = TranspilerContext {
            names: &names,
            functions_by_id: &functions_by_id,
            builtins: &builtins,
            expressions: &implementation.expression_forest,
            types: &implementation.type_forest
        };

        transpile_function(stream, implementation, &context).unwrap();
    }

    writeln!(stream, "\n\n__all__ = [")?;
    for function in exported_symbols.iter() {
        writeln!(stream, "    \"{}\",", &names[&function.pointer.pointer_id])?;
    }
    writeln!(stream, "]")?;

    if let Some(main_function) = find_annotated(exported_symbols.iter(), "main") {
        write!(stream, "\n\nif __name__ == \"__main__\":\n    {}()\n", names.get(&main_function.pointer.pointer_id).unwrap())?;
    }

    return Ok(())
}

pub fn transpile_function(stream: &mut (dyn Write), function: &FunctionImplementation, context: &TranspilerContext) -> Result<(), std::io::Error> {
    write!(stream, "\n\ndef {}(", context.names[&function.pointer.pointer_id])?;

    for (idx, parameter) in function.parameter_variables.iter().enumerate() {
        write!(stream, "{}: ", context.names.get(&parameter.id).unwrap())?;
        types::transpile(stream, &parameter.type_, context)?;
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

    for (idx, parameter) in function.parameter_variables.iter().enumerate() {
        let variable_name = context.names.get(&parameter.id).unwrap();
        let external_name = variable_name;  // external names are not supported in python

        match &parameter.type_.unit {
            TypeUnit::Monad => {
                let unit = &parameter.type_.arguments[0].as_ref().unit;

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
        ExpressionOperation::FunctionCall(call) => {
            let pointer = &call.pointer;
            let resolution = &call.requirements_fulfillment;
            let arguments = context.expressions.arguments.get(&expression).unwrap();

            if
                try_transpile_keyword(stream, pointer, context)?
                || try_transpile_binary_operator(stream, pointer, arguments, context)?
                || try_transpile_constant(stream, pointer, arguments, &expression, context)?
                || try_transpile_unary_operator(stream, pointer, arguments, context)?
                || try_transpile_literal(stream, pointer, arguments, &expression, context)?
            {
                // no-op
            }
            else {
                match &pointer.call_type {
                    // Can reference the static function
                    FunctionCallType::Static => {
                        guard!(let Some(name) = context.names.get(&pointer.pointer_id) else {
                            panic!("Couldn't find name in python: {:?}", pointer)
                        });
                        write!(stream, "{}", name)?
                    },
                    // Have to reference the function by trait
                    FunctionCallType::Polymorphic { requirement, abstract_function } => {
                        todo!("Polymorphic calls should have been unfolded earlier. Python generics functionality can be restored later. {:?}", pointer)
                        // write!(stream, "{}.{}", &context.names[todo!("We used to look for 'declaration ID', but that was weird, where is the name stored?")], context.names[&pointer.pointer_id])?;
                    }
                }
                write!(stream, "(")?;

                // TODO Only required when we're forward-passing unresolved requirements
                let requirements: [&Rc<TraitBinding>; 0] = [];  // function.target.interface.requirements
                let mut arguments_left = arguments.len() + requirements.len();

                for (parameter, argument) in zip_eq(pointer.target.interface.parameters.iter(), arguments.iter()) {
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
                    todo!()
                    // let implementation = &context.functions_by_id[&pointer.pointer_id];
                    // let declaration = &implementation.conformance_delegations[requirement];
                    //
                    // let param_name = context.names.get(&declaration.id).unwrap();
                    // let arg_name = context.names.get(todo!()).unwrap();
                    // write!(stream, "{}={}", param_name, arg_name)?;
                    //
                    // arguments_left -= 1;
                    // if arguments_left > 0 {
                    //     write!(stream, ", ")?;
                    // }
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
        ExpressionOperation::PairwiseOperations { calls } => {
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
