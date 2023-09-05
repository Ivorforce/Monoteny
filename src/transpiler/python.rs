pub mod docstrings;
pub mod types;
pub mod builtins;
pub mod class;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
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
use crate::program::functions::{FunctionHead, FunctionType, ParameterKey};
use crate::program::{find_annotated, Program};
use crate::program::calls::FunctionBinding;
use crate::program::generics::TypeForest;
use crate::program::global::{BuiltinFunctionHint, FunctionImplementation, PrimitiveOperation};
use crate::program::traits::{RequirementsFulfillment, TraitBinding};
use crate::program::types::{TypeProto, TypeUnit};
use crate::transpiler::namespaces;
use crate::transpiler::python::class::{ClassContext, transpile_class};

pub struct FunctionContext<'a> {
    names: &'a HashMap<Uuid, String>,
    functions_by_id: &'a HashMap<Uuid, Rc<FunctionImplementation>>,
    builtins: &'a Builtins,
    builtin_hints: &'a HashMap<Uuid, BuiltinFunctionHint>,
    transpilation_hints: &'a HashMap<Uuid, TranspilationHint>,
    expressions: &'a ExpressionForest,
    types: &'a TypeForest,
    struct_ids: &'a HashMap<Box<TypeProto>, Uuid>,
}

#[derive(PartialEq, Eq, Clone)]
enum TranspilationHint {
    CallProvided(String)
}

const MATH_REPLACEMENTS: [(&str, &str); 17] = [
    ("factorial", "math.factorial"),

    ("sin", "math.sin"),
    ("cos", "math.cos"),
    ("tan", "math.tan"),
    ("sinh", "math.sinh"),
    ("cosh", "math.cosh"),
    ("tanh", "math.tanh"),
    ("arcsin", "math.asin"),
    ("arccos", "math.acos"),
    ("arctan", "math.atan"),
    ("arcsinh", "math.asinh"),
    ("arccosh", "math.acosh"),
    ("arctanh", "math.atanh"),

    ("ceil", "math.ceil"),
    ("floor", "math.floor"),
    ("round", "round"),

    ("abs", "abs"),
];

pub fn transpile_program(stream: &mut (dyn Write), program: &Program, builtins: &Rc<Builtins>) -> Result<(), std::io::Error> {
    let mut struct_ids = HashMap::new();

    let mut global_namespace = builtins::create(&builtins, &mut struct_ids);
    let builtin_structs: HashSet<_> = struct_ids.keys().map(Clone::clone).collect();
    let mut file_namespace = global_namespace.add_sublevel();
    let mut object_namespace = namespaces::Level::new();  // TODO Keywords can't be in object namespace either

    let mut functions_by_id = HashMap::new();
    let mut builtin_hints_by_id = HashMap::new();
    let mut transpilation_hints_by_id: HashMap<Uuid, TranspilationHint> = HashMap::new();
    let mut pointer_by_id = HashMap::new();

    for module in [&program.module].into_iter().chain(builtins.all_modules()) {
        for implementation in module.function_implementations.values() {
            functions_by_id.insert(implementation.function_id, Rc::clone(implementation));
        }
        for (head, hint) in module.builtin_hints.iter() {
            builtin_hints_by_id.insert(head.function_id, hint.clone());
        }
        for pointer in module.function_pointers.values() {
            pointer_by_id.insert(pointer.target.function_id, Rc::clone(pointer));
        }
    }

    let math_replacements: HashMap<_, _> = MATH_REPLACEMENTS.iter()
        .map(|(src, dst)| (src.to_string(), dst.to_string()))
        .collect();
    for ptr in builtins.module_by_name["math".into()].function_pointers.values() {
        if let Some(fn_name) = math_replacements.get(&ptr.name) {
            let id: Uuid = ptr.target.function_id;
            let hint: TranspilationHint = TranspilationHint::CallProvided(fn_name.clone());
            transpilation_hints_by_id.insert(id, hint);
        }
    }

    let exported_symbols: Rc<RefCell<Vec<Rc<FunctionImplementation>>>> = Rc::new(RefCell::new(vec![]));
    let unfolder: Rc<RefCell<FunctionUnfolder>> = Rc::new(RefCell::new(FunctionUnfolder::new()));

    fn should_unfold(f: &Rc<FunctionBinding>, primitives: &HashMap<Uuid, BuiltinFunctionHint>, transpilation_hints_by_id: &HashMap<Uuid, TranspilationHint>) -> bool {
        if primitives.contains_key(&f.function.function_id) {
            // We need to inject these
            return false;
        }

        if transpilation_hints_by_id.contains_key(&f.function.function_id) {
            // We want to inject / override these
            return false;
        }

        true
    }

    // Run interpreter

    interpreter::run::transpile(program, &Rc::clone(&builtins), &|implementation| {
        if implementation.head.interface.collect_generics().len() > 0 {
            // We'll need to somehow transpile requirements as protocols and generics as generics.
            // That's for later!
            panic!("Transpiling generic functions is not supported yet: {:?}", implementation.head);
        }

        let unfolded_function = unfolder.borrow_mut().deref_mut().unfold_anonymous(
            implementation,
            &Rc::new(FunctionBinding {
                // The implementation's pointer is fine.
                function: Rc::clone(&implementation.head),
                // The resolution SHOULD be empty: The function is transpiled WITH its generics.
                // Unless generics are bound in the transpile directive, which is TODO
                requirements_fulfillment: RequirementsFulfillment::empty(),
            }),
            &|f| should_unfold(f, &builtin_hints_by_id, &transpilation_hints_by_id)
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
        let implementation = &functions_by_id[&used_symbol.function.function_id];

        let unfolded_implementation = unfolder.unfold_anonymous(
            implementation,
            &replacement_symbol,
            &|f| should_unfold(f, &builtin_hints_by_id, &transpilation_hints_by_id)
        );

        internal_symbols.push(unfolded_implementation);
    }

    for implementation in exported_symbols.iter() {
        // TODO Register with priority over internal symbols
        // TODO Pointers (i.e. function name and form) were collected with heads
        //  earlier, but after unfolding we have different heads. We should find
        //  _one_ fitting pointer for each head (although there may be more than one?)
        //  and use its function name.
        file_namespace.register_definition(implementation.head.function_id, &"fun".into());
    }

    for implementation in internal_symbols.iter() {
        file_namespace.register_definition(implementation.head.function_id, &"fun".into());
    }

    for implementation in exported_symbols.iter().chain(internal_symbols.iter()) {
        let function_namespace = file_namespace.add_sublevel();
        for (variable, name) in implementation.variable_names.iter() {
            function_namespace.register_definition(variable.id.clone(), name);
        }
        for (expression_id, operation) in implementation.expression_forest.operations.iter() {
            if let ExpressionOperation::FunctionCall(fun) = operation {
                if let Some(BuiltinFunctionHint::Constructor) = builtin_hints_by_id.get(&fun.function.function_id) {
                    let type_ = implementation.type_forest.resolve_binding_alias(expression_id).unwrap();
                    if let Entry::Vacant(entry) = struct_ids.entry(type_.clone()) {
                        // TODO If we have generics, we should include their bindings in the name somehow.
                        //  Eg. ArrayFloat. Probably only if it's exactly one. Otherwise, we need to be ok with
                        //  just the auto-renames.
                        let name = match &type_.unit {
                            TypeUnit::Struct(struct_) => &struct_.name,
                            // Technically only the name is unsupported here, but later we'd need to actually construct it too.
                            _ => panic!("Unsupported Constructor Type")
                        };
                        let id = Uuid::new_v4();
                        entry.insert(id);
                        // TODO Find proper names
                        file_namespace.register_definition(id, name);
                    }
                }
            }
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

    for (struct_type, id) in struct_ids.iter() {
        if builtin_structs.contains(struct_type) {
            continue
        }

        let context = ClassContext {
            names: &names,
            functions_by_id: &functions_by_id,
            builtins: &builtins,
            builtin_hints: &builtin_hints_by_id,
            struct_ids: &struct_ids,
        };

        transpile_class(stream, struct_type, &context)?;
    }

    for implementation in exported_symbols.iter() {
        let context = FunctionContext {
            names: &names,
            functions_by_id: &functions_by_id,
            builtins: &builtins,
            builtin_hints: &builtin_hints_by_id,
            transpilation_hints: &transpilation_hints_by_id,
            expressions: &implementation.expression_forest,
            types: &implementation.type_forest,
            struct_ids: &struct_ids,
        };

        transpile_function(stream, implementation, &context).unwrap();
    }

    write!(stream, "\n\n")?;
    writeln!(stream, "# ========================== ======== ============================")?;
    writeln!(stream, "# ========================== Internal ============================")?;
    writeln!(stream, "# ========================== ======== ============================")?;

    for implementation in internal_symbols.iter() {
        let context = FunctionContext {
            names: &names,
            functions_by_id: &functions_by_id,
            builtins: &builtins,
            builtin_hints: &builtin_hints_by_id,
            transpilation_hints: &transpilation_hints_by_id,
            expressions: &implementation.expression_forest,
            types: &implementation.type_forest,
            struct_ids: &struct_ids,
        };

        transpile_function(stream, implementation, &context).unwrap();
    }

    writeln!(stream, "\n\n__all__ = [")?;
    for function in exported_symbols.iter() {
        writeln!(stream, "    \"{}\",", &names[&function.head.function_id])?;
    }
    writeln!(stream, "]")?;

    if let Some(main_function) = find_annotated(exported_symbols.iter(), "main") {
        write!(stream, "\n\nif __name__ == \"__main__\":\n    {}()\n", names.get(&main_function.head.function_id).unwrap())?;
    }

    return Ok(())
}

pub fn transpile_function(stream: &mut (dyn Write), function: &FunctionImplementation, context: &FunctionContext) -> Result<(), std::io::Error> {
    write!(stream, "\n\ndef {}(", context.names[&function.head.function_id])?;

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

    if !function.head.interface.return_type.unit.is_void() {
        write!(stream, " -> ", )?;
        types::transpile(stream, &function.head.interface.return_type, context)?;
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
                let type_ = &parameter.type_.arguments[0].as_ref();

                if let TypeUnit::Struct(s) = &type_.unit {
                    write!(
                        stream, "    {} = np.asarray({}, dtype=",
                        variable_name,
                        external_name
                    )?;
                    types::transpile(stream, type_, context)?;
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

pub fn transpile_expression(stream: &mut (dyn Write), expression: ExpressionID, context: &FunctionContext) -> Result<(), std::io::Error> {
    match &context.expressions.operations.get(&expression).unwrap() {
        ExpressionOperation::StringLiteral(string) => {
            write!(stream, "\"{}\"", escape_string(&string))?;
        }
        ExpressionOperation::VariableLookup(variable) => {
            let variable_name = context.names.get(&variable.id).unwrap();

            write!(stream, "{}", variable_name)?;
        }
        ExpressionOperation::FunctionCall(call) => {
            let function = &call.function;
            let resolution = &call.requirements_fulfillment;
            let arguments = context.expressions.arguments.get(&expression).unwrap();

            if
                try_transpile_optimization(stream, function, arguments, &expression, context)?
                || try_transpile_builtin(stream, function, &expression, arguments, context)?
            {
                // no-op
            }
            else {
                match &function.function_type {
                    // Can reference the static function
                    FunctionType::Static => {
                        guard!(let Some(name) = context.names.get(&function.function_id) else {
                            panic!("Couldn't find name in python: {:?}", function)
                        });
                        write!(stream, "{}", name)?
                    },
                    // Have to reference the function by trait
                    FunctionType::Polymorphic { requirement, abstract_function } => {
                        todo!("Polymorphic calls should have been unfolded earlier. Python generics functionality can be restored later. {:?}", function)
                        // write!(stream, "{}.{}", &context.names[todo!("We used to look for 'declaration ID', but that was weird, where is the name stored?")], context.names[&pointer.pointer_id])?;
                    }
                }
                write!(stream, "(")?;

                // TODO Only required when we're forward-passing unresolved requirements
                let requirements: [&Rc<TraitBinding>; 0] = [];  // function.target.interface.requirements
                let mut arguments_left = arguments.len() + requirements.len();

                for (parameter, argument) in zip_eq(function.interface.parameters.iter(), arguments.iter()) {
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

pub fn transpile_maybe_parenthesized_expression(stream: &mut (dyn Write), expression: ExpressionID, context: &FunctionContext) -> Result<(), std::io::Error> {
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

pub fn try_transpile_builtin(stream: &mut (dyn Write), function: &Rc<FunctionHead>, expression_id: &ExpressionID, arguments: &Vec<ExpressionID>, context: &FunctionContext) -> Result<bool, std::io::Error> {
    guard!(let Some(hint) = context.builtin_hints.get(&function.function_id) else {
        return Ok(false);
    });

    match hint {
        BuiltinFunctionHint::PrimitiveOperation { type_, operation } => {
            match operation {
                PrimitiveOperation::And => transpile_binary_operator(stream, "and", arguments, context)?,
                PrimitiveOperation::Or => transpile_binary_operator(stream, "or", arguments, context)?,
                PrimitiveOperation::Not => transpile_unary_operator(stream, "not ", arguments, context)?,
                PrimitiveOperation::Negative => transpile_unary_operator(stream, "-", arguments, context)?,
                PrimitiveOperation::Add => transpile_binary_operator(stream, "+", arguments, context)?,
                PrimitiveOperation::Subtract => transpile_binary_operator(stream, "-", arguments, context)?,
                PrimitiveOperation::Multiply => transpile_binary_operator(stream, "*", arguments, context)?,
                // TODO This should be truediv for ints
                PrimitiveOperation::Divide => transpile_binary_operator(stream, "/", arguments, context)?,
                PrimitiveOperation::Modulo => transpile_binary_operator(stream, "%", arguments, context)?,
                PrimitiveOperation::Exp => transpile_binary_operator(stream, "**", arguments, context)?,
                PrimitiveOperation::Log => transpile_single_arg_function_call(stream, "math.log", arguments, expression_id, context)?,
                PrimitiveOperation::EqualTo => transpile_binary_operator(stream, "==", arguments, context)?,
                PrimitiveOperation::NotEqualTo => transpile_binary_operator(stream, "!=", arguments, context)?,
                PrimitiveOperation::GreaterThan => transpile_binary_operator(stream, ">", arguments, context)?,
                PrimitiveOperation::LesserThan => transpile_binary_operator(stream, "<", arguments, context)?,
                PrimitiveOperation::GreaterThanOrEqual => transpile_binary_operator(stream, ">=", arguments, context)?,
                PrimitiveOperation::LesserThanOrEqual => transpile_binary_operator(stream, "<=", arguments, context)?,
                PrimitiveOperation::ParseIntString => transpile_parse_function(stream, "^[0-9]+$", arguments, expression_id, context)?,
                PrimitiveOperation::ParseFloatString => transpile_parse_function(stream, "^[0-9]+\\.[0-9]*$", arguments, expression_id, context)?,
            }
        }
        BuiltinFunctionHint::Constructor => {
            let struct_type = context.types.resolve_binding_alias(expression_id).unwrap();
            let struct_id = context.struct_ids[&struct_type];
            // TODO need to pass in parameters once they exist
            write!(stream, "{}()", context.names[&struct_id])?
        },
        BuiltinFunctionHint::True => write!(stream, "True")?,
        BuiltinFunctionHint::False => write!(stream, "False")?,
        BuiltinFunctionHint::Print => transpile_single_arg_function_call(stream, "print", arguments, expression_id, context)?,
        BuiltinFunctionHint::Panic => write!(stream, "exit(1)")?,
    }

    return Ok(true)
}

pub fn transpile_unary_operator(stream: &mut (dyn Write), operator: &str, arguments: &Vec<ExpressionID>, context: &FunctionContext) -> Result<(), std::io::Error> {
    guard!(let [expression] = arguments[..] else {
        panic!("Unary operator got {} arguments: {}", arguments.len(), operator);
    });

    write!(stream, "{}", operator)?;
    transpile_maybe_parenthesized_expression(stream, expression.clone(), context)
}

pub fn transpile_binary_operator(stream: &mut (dyn Write), operator: &str, arguments: &Vec<ExpressionID>, context: &FunctionContext) -> Result<(), std::io::Error> {
    guard!(let [lhs, rhs] = arguments[..] else {
        panic!("Binary operator got {} arguments: {}", arguments.len(), operator);
    });

    transpile_maybe_parenthesized_expression(stream, lhs.clone(), context)?;
    write!(stream, " {} ", operator)?;
    transpile_maybe_parenthesized_expression(stream, rhs.clone(), context)
}

pub fn try_transpile_optimization(stream: &mut (dyn Write), function: &Rc<FunctionHead>, arguments: &Vec<ExpressionID>, expression_id: &ExpressionID, context: &FunctionContext) -> Result<bool, std::io::Error> {
    if !context.builtins.module_by_name["math".into()].functions_references.contains_key(function) {
        return Ok(false)
    }

    if let Some(transpilation_hint) = context.transpilation_hints.get(&function.function_id) {
        match transpilation_hint {
            TranspilationHint::CallProvided(python_name) => {
                write!(stream, "{}(", python_name)?;
                for (idx, expression) in arguments.iter().enumerate() {
                    transpile_expression(stream, *expression, context)?;

                    if idx < arguments.len() - 1 {
                        write!(stream, ", ")?;
                    }
                }
                write!(stream, ")")?;
            }
        }

        return Ok(true)
    }

    Ok(false)
}

pub fn transpile_parse_function(stream: &mut (dyn Write), supported_regex: &str, arguments: &Vec<ExpressionID>, expression_id: &ExpressionID, context: &FunctionContext) -> Result<(), std::io::Error> {
    guard!(let [argument_expression_id] = arguments[..] else {
        panic!("Parse function got {} arguments", arguments.len());
    });

    types::transpile(stream, &context.types.resolve_binding_alias(expression_id).unwrap(), context)?;

    if let ExpressionOperation::StringLiteral(literal) = &context.expressions.operations[&argument_expression_id] {
        let is_supported_literal = regex::Regex::new(supported_regex).unwrap();
        if is_supported_literal.is_match(literal) {
            return write!(stream, "({})",  literal)
        }
    }

    write!(stream, "(", )?;
    transpile_expression(stream, argument_expression_id, context)?;
    write!(stream, ")", )
}

pub fn transpile_single_arg_function_call(stream: &mut (dyn Write), function_name: &str, arguments: &Vec<ExpressionID>, expression_id: &ExpressionID, context: &FunctionContext) -> Result<(), std::io::Error> {
    guard!(let [argument_expression_id] = arguments[..] else {
        panic!("{} function got {} arguments", function_name, arguments.len());
    });

    write!(stream, "{}(", function_name)?;
    transpile_expression(stream, argument_expression_id, context)?;
    write!(stream, ")", )
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
