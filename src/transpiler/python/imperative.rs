use std::collections::HashMap;
use std::rc::Rc;
use guard::guard;
use itertools::zip_eq;
use uuid::Uuid;
use regex;
use crate::interpreter::Runtime;
use crate::program::computation_tree::*;
use crate::program::functions::{FunctionHead, FunctionType, ParameterKey};
use crate::program::generics::TypeForest;
use crate::program::global::{BuiltinFunctionHint, FunctionImplementation, PrimitiveOperation};
use crate::program::traits::TraitBinding;
use crate::program::types::TypeProto;
use crate::transpiler::python::optimization::{TranspilationHint, try_transpile_optimization};
use crate::transpiler::python::{ast, types};

pub struct FunctionContext<'a> {
    pub names: &'a HashMap<Uuid, String>,
    pub struct_ids: &'a HashMap<Box<TypeProto>, Uuid>,

    pub runtime: &'a Runtime,
    pub fn_transpilation_hints: &'a HashMap<Rc<FunctionHead>, TranspilationHint>,

    pub expressions: &'a ExpressionForest,
    pub types: &'a TypeForest,
}

pub fn transpile_function(function: &FunctionImplementation, context: &FunctionContext) -> Box<ast::Function> {
    let mut syntax = Box::new(ast::Function {
        name: context.names[&function.head.function_id].clone(),
        parameters: function.parameter_variables.iter().map(|parameter| {
            Box::new(ast::Parameter {
                name: context.names[&parameter.id].clone(),
                type_: types::transpile(&parameter.type_, context),
            })
        }).collect(),
        return_type: match function.head.interface.return_type.unit.is_void() {
            true => None,
            false => Some(types::transpile(&function.head.interface.return_type, context))
        },
        statements: vec![],
    });

    // TODO We only need this when we do monads again
    // for parameter in function.parameter_variables.iter() {
    //     let variable_name = context.names.get(&parameter.id).unwrap();
    //     let external_name = variable_name;  // external names are not supported in python
    //
    //     match &parameter.type_.unit {
    //         TypeUnit::Monad => {
    //             let type_ = &parameter.type_.arguments[0].as_ref();
    //
    //             if let TypeUnit::Struct(s) = &type_.unit {
    //                 syntax.statements.push(Box::new(tree::Statement {
    //                     contents: format!(
    //                         "{} = np.asarray({}, dtype={})",
    //                         variable_name,
    //                         external_name,
    //                         types::transpile(type_, context),
    //                     ),
    //                 }));
    //             }
    //             else {
    //                 panic!("Can't have a nested monad in numpy.")
    //             }
    //         }
    //         _ => {}
    //     };
    // }

    for statement in function.statements.iter() {
        syntax.statements.push(Box::new(match statement.as_ref() {
                Statement::Return(value) => {
                    ast::Statement::Return(value.map(|value| transpile_expression(value.clone(), context)))
                }
                Statement::VariableAssignment(variable, expression) => {
                    ast::Statement::VariableAssignment {
                        variable_name: context.names[&variable.id].clone(),
                        value: transpile_expression(expression.clone(), context),
                    }
                }
                Statement::Expression(expression) => {
                    ast::Statement::Expression(transpile_expression(expression.clone(), context))
                }
            }
        ));
    }

    syntax
}

pub fn transpile_expression(expression: ExpressionID, context: &FunctionContext) -> Box<ast::Expression> {
    match &context.expressions.operations.get(&expression).unwrap() {
        ExpressionOperation::StringLiteral(string) => {
            Box::new(ast::Expression::StringLiteral(string.clone()))
        }
        ExpressionOperation::VariableLookup(variable) => {
            Box::new(ast::Expression::VariableLookup(context.names[&variable.id].clone()))
        }
        ExpressionOperation::FunctionCall(call) => {
            let function = &call.function;
            let resolution = &call.requirements_fulfillment;
            let arguments = context.expressions.arguments.get(&expression).unwrap();

            if let Some(s) = try_transpile_optimization(function, arguments, &expression, context) {
                s
            }
            else if let Some(s) = try_transpile_builtin(function, &expression, arguments, context) {
                s
            }
            else {
                let function_name = match &function.function_type {
                    // Can reference the static function
                    FunctionType::Static => {
                        guard!(let Some(name) = context.names.get(&function.function_id) else {
                            panic!("Couldn't find name for function: {:?}", function)
                        });
                        name.clone()
                    },
                    // Have to reference the function by trait
                    FunctionType::Polymorphic { provided_by_assumption, abstract_function } => {
                        todo!("Polymorphic calls (from generic transpilations) are not supported yet: {:?}", function)
                    }
                };

                let mut py_arguments = vec![];

                for (parameter, argument) in zip_eq(function.interface.parameters.iter(), arguments.iter()) {
                    py_arguments.push((parameter.external_key.clone(), transpile_expression(argument.clone(), context)));
                }

                // TODO Only required when we're forward-passing unresolved requirements
                let requirements: [&Rc<TraitBinding>; 0] = [];  // function.target.interface.requirements
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

                Box::new(ast::Expression::FunctionCall(function_name, py_arguments))
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
        ExpressionOperation::Block(_) => {
            todo!()
        }
    }
}

pub fn escape_string(string: &str) -> String {
    // This is kinda sad lol. There's gotta be a better way.
    // FIXME This will also wreck something like \\n.
    let string = string.replace("\\", "\\\\");
    let string = string.replace("\n", "\\n");
    let string = string.replace("\0", "\\0");
    let string = string.replace("\t", "\\t");
    let string = string.replace("\r", "\\r");
    let string = string.replace("\"", "\\\"");
    return string
}

pub fn try_transpile_builtin(function: &Rc<FunctionHead>, expression_id: &ExpressionID, arguments: &Vec<ExpressionID>, context: &FunctionContext) -> Option<Box<ast::Expression>> {
    guard!(let Some(hint) = context.runtime.source.fn_builtin_hints.get(function) else {
        return None;
    });

    Some(match hint {
        // TODO Many of these operations automatically 'upgrade' the type.
        BuiltinFunctionHint::PrimitiveOperation { type_, operation } => {
            match operation {
                PrimitiveOperation::And => transpile_binary_operator("and", arguments, context),
                PrimitiveOperation::Or => transpile_binary_operator("or", arguments, context),
                PrimitiveOperation::Not => transpile_unary_operator("not ", arguments, context),
                PrimitiveOperation::Negative => transpile_unary_operator("-", arguments, context),
                PrimitiveOperation::Add => transpile_binary_operator("+", arguments, context),
                PrimitiveOperation::Subtract => transpile_binary_operator("-", arguments, context),
                PrimitiveOperation::Multiply => transpile_binary_operator("*", arguments, context),
                PrimitiveOperation::Divide => {
                    if type_.is_int() {
                        transpile_binary_operator("//", arguments, context)
                    }
                    else {
                        transpile_binary_operator("/", arguments, context)
                    }
                },
                PrimitiveOperation::Modulo => transpile_binary_operator("%", arguments, context),
                PrimitiveOperation::Exp => transpile_binary_operator("**", arguments, context),
                PrimitiveOperation::Log => transpile_single_arg_function_call("math.log", arguments, expression_id, context),
                PrimitiveOperation::EqualTo => transpile_binary_operator("==", arguments, context),
                PrimitiveOperation::NotEqualTo => transpile_binary_operator("!=", arguments, context),
                PrimitiveOperation::GreaterThan => transpile_binary_operator(">", arguments, context),
                PrimitiveOperation::LesserThan => transpile_binary_operator("<", arguments, context),
                PrimitiveOperation::GreaterThanOrEqual => transpile_binary_operator(">=", arguments, context),
                PrimitiveOperation::LesserThanOrEqual => transpile_binary_operator("<=", arguments, context),
                PrimitiveOperation::ParseIntString => transpile_parse_function("^[0-9]+$", arguments, expression_id, context),
                PrimitiveOperation::ParseFloatString => transpile_parse_function("^[0-9]+\\.[0-9]*$", arguments, expression_id, context),
                PrimitiveOperation::ToString => transpile_single_arg_function_call("str", arguments, expression_id, context),
            }
        }
        BuiltinFunctionHint::Constructor => {
            let struct_type = context.types.resolve_binding_alias(expression_id).unwrap();
            let struct_id = context.struct_ids[&struct_type];
            // TODO need to pass in parameters once they exist
            Box::new(ast::Expression::FunctionCall(
                context.names[&struct_id].clone(),
                vec![]
            ))
        },
        BuiltinFunctionHint::True => Box::new(ast::Expression::ValueLiteral("True".to_string())),
        BuiltinFunctionHint::False => Box::new(ast::Expression::ValueLiteral("False".to_string())),
    })
}

pub fn transpile_unary_operator(operator: &str, arguments: &Vec<ExpressionID>, context: &FunctionContext) -> Box<ast::Expression> {
    guard!(let [expression] = arguments[..] else {
        panic!("Unary operator got {} arguments: {}", arguments.len(), operator);
    });

    Box::new(ast::Expression::UnaryOperation(operator.to_string(), transpile_expression(expression, context)))
}

pub fn transpile_binary_operator(operator: &str, arguments: &Vec<ExpressionID>, context: &FunctionContext) -> Box<ast::Expression> {
    guard!(let [lhs, rhs] = arguments[..] else {
        panic!("Binary operator got {} arguments: {}", arguments.len(), operator);
    });

    Box::new(ast::Expression::BinaryOperation(transpile_expression(lhs, context), operator.to_string(), transpile_expression(rhs, context)))
}

pub fn transpile_parse_function(supported_regex: &str, arguments: &Vec<ExpressionID>, expression_id: &ExpressionID, context: &FunctionContext) -> Box<ast::Expression> {
    guard!(let [argument_expression_id] = arguments[..] else {
        panic!("Parse function got {} arguments", arguments.len());
    });

    let value = match &context.expressions.operations[&argument_expression_id] {
        ExpressionOperation::StringLiteral(literal) => {
            let is_supported_literal = regex::Regex::new(supported_regex).unwrap();
            if is_supported_literal.is_match(literal) {
                Box::new(ast::Expression::ValueLiteral(literal.clone()))
            }
            else {
                transpile_expression(argument_expression_id, context)
            }
        }
        _ => transpile_expression(argument_expression_id, context),
    };

    Box::new(ast::Expression::FunctionCall(
        types::transpile(&context.types.resolve_binding_alias(expression_id).unwrap(), context),
        vec![(ParameterKey::Positional, value)]
    ))
}

pub fn transpile_single_arg_function_call(function_name: &str, arguments: &Vec<ExpressionID>, expression_id: &ExpressionID, context: &FunctionContext) -> Box<ast::Expression> {
    guard!(let [argument_expression_id] = arguments[..] else {
        panic!("{} function got {} arguments", function_name, arguments.len());
    });

    Box::new(ast::Expression::FunctionCall(
        function_name.to_string(),
        vec![(ParameterKey::Positional, transpile_expression(argument_expression_id, context))]
    ))
}

pub fn is_simple(operation: &ExpressionOperation) -> bool {
    match operation {
        ExpressionOperation::VariableLookup(_) => true,
        ExpressionOperation::StringLiteral(_) => true,
        ExpressionOperation::ArrayLiteral => true,
        ExpressionOperation::FunctionCall { .. } => false,
        ExpressionOperation::PairwiseOperations { .. } => false,
        ExpressionOperation::Block(_) => todo!(),
    }
}