use std::collections::HashMap;
use std::rc::Rc;
use guard::guard;
use itertools::{Either, Itertools, zip_eq};
use itertools::Either::{Left, Right};
use uuid::Uuid;
use regex;
use crate::program::expression_tree::*;
use crate::program::functions::{FunctionHead, ParameterKey};
use crate::program::generics::TypeForest;
use crate::program::global::{FunctionLogicDescriptor, FunctionImplementation, PrimitiveOperation};
use crate::transpiler::python::{ast, types};
use crate::transpiler::python::representations::{FunctionForm, Representations};
use crate::util::strings;

pub struct FunctionContext<'a> {
    pub names: &'a HashMap<Uuid, String>,

    pub representations: &'a Representations,
    pub logic: &'a HashMap<Rc<FunctionHead>, FunctionLogicDescriptor>,

    pub expressions: &'a ExpressionTree,
    pub types: &'a TypeForest,
}

pub fn transpile_function(implementation: &FunctionImplementation, context: &FunctionContext) -> Box<ast::Statement> {
    match &context.representations.function_representations[&implementation.head] {
        FunctionForm::Constant(id) => {
            Box::new(ast::Statement::VariableAssignment {
                target: Box::new(ast::Expression::NamedReference(context.names[id].clone())),
                value: Some(transpile_expression(implementation.expression_tree.root, context)),
                type_annotation: Some(types::transpile(&implementation.head.interface.return_type, context)),
            })
        }
        FunctionForm::FunctionCall(id) => Box::new(ast::Statement::Function(transpile_plain_function(implementation, context.names[id].clone(), context))),
        FunctionForm::CallAsFunction => panic!(),
        FunctionForm::GetMemberField(id) => panic!(),
        FunctionForm::SetMemberField(_) => panic!(),
        FunctionForm::MemberCall(id) => panic!(),
        FunctionForm::Unary(id) => panic!("Internal Error: Custom static unary functions are not supported in python"),
        FunctionForm::Binary(id) => panic!("Internal Error: Custom static binary functions are not supported in python"),
    }
}

pub fn transpile_plain_function(implementation: &FunctionImplementation, name: String, context: &FunctionContext) -> Box<ast::Function> {
    let mut syntax = Box::new(ast::Function {
        name,
        parameters: implementation.parameter_locals.iter().map(|parameter| {
            Box::new(ast::Parameter {
                name: context.names[&parameter.id].clone(),
                type_: types::transpile(&parameter.type_, context),
            })
        }).collect(),
        return_type: match implementation.head.interface.return_type.unit.is_void() {
            true => None,
            false => Some(types::transpile(&implementation.head.interface.return_type, context))
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

    syntax.statements = match &implementation.expression_tree.values[&implementation.expression_tree.root] {
        ExpressionOperation::Block => {
            transpile_block(&implementation, context, &implementation.expression_tree.children[&implementation.expression_tree.root])
        }
        _ => {
            let expression = transpile_expression(implementation.expression_tree.root, context);

            vec![Box::new(match implementation.head.interface.return_type.unit.is_void() {
                true => ast::Statement::Expression(expression),
                false => ast::Statement::Return(Some(expression)),
            })]
        }
    };

    syntax
}

fn transpile_block(implementation: &&FunctionImplementation, context: &FunctionContext, statements: &Vec<ExpressionID>) -> Vec<Box<ast::Statement>> {
    let mut statements_ = vec![];

    for statement in statements.iter() {
        let operation = &implementation.expression_tree.values[&statement];
        statements_.push(match operation {
            ExpressionOperation::Block => todo!(),
            ExpressionOperation::SetLocal(variable) => {
                Box::new(ast::Statement::VariableAssignment {
                    target: Box::new(ast::Expression::NamedReference(context.names[&variable.id].clone())),
                    value: Some(transpile_expression(implementation.expression_tree.children[&statement][0], context)),
                    // TODO We can omit the type annotation if we assign the variable a second time
                    type_annotation: Some(types::transpile(&variable.type_, context)),
                })
            }
            ExpressionOperation::Return => {
                let value = implementation.expression_tree.children[&statement].iter().exactly_one().ok();
                Box::new(ast::Statement::Return(value.map(|value| transpile_expression(*value, context))))
            }
            ExpressionOperation::FunctionCall(call) => {
                match transpile_function_call(context, &call.function, &context.representations.function_representations[&call.function], *statement) {
                    Left(e) => Box::new(ast::Statement::Expression(e)),
                    Right(s) => s,
                }
            }
            _ => Box::new(ast::Statement::Expression(transpile_expression(*statement, context))),
        });
    }

    statements_
}

pub fn transpile_expression(expression_id: ExpressionID, context: &FunctionContext) -> Box<ast::Expression> {
    match &context.expressions.values.get(&expression_id).unwrap() {
        ExpressionOperation::StringLiteral(string) => {
            Box::new(ast::Expression::StringLiteral(string.clone()))
        }
        ExpressionOperation::GetLocal(variable) => {
            Box::new(ast::Expression::NamedReference(context.names[&variable.id].clone()))
        }
        ExpressionOperation::FunctionCall(call) => {
            match transpile_function_call(context, &call.function, &context.representations.function_representations[&call.function], expression_id) {
                Left(e) => e,
                Right(s) => panic!("Statement not supported in expression context.")
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
        ExpressionOperation::Block => todo!(),
        ExpressionOperation::SetLocal(_) => panic!("Variable assignment not allowed as expression."),
        ExpressionOperation::Return => panic!("Return not allowed as expression."),
    }
}

fn transpile_function_call(context: &FunctionContext, function: &Rc<FunctionHead>, form: &FunctionForm, expression_id: ExpressionID) -> Either<Box<ast::Expression>, Box<ast::Statement>> {
    let arguments = context.expressions.children.get(&expression_id).unwrap();

    if let Some(s) = try_transpile_optimization(function, &expression_id, arguments, context) {
        return Left(s)
    }

    let mut py_arguments = vec![];
    let mut arguments = arguments.clone();
    let mut parameters = function.interface.parameters.clone();

    let target = match form {
        FunctionForm::Constant(id) => {
            assert!(arguments.is_empty());
            return Left(Box::new(ast::Expression::NamedReference(context.names[id].clone())))
        },
        FunctionForm::Unary(id) => return Left(transpile_unary_operator(&context.names[&id], &arguments, context)),
        FunctionForm::Binary(id) => return Left(transpile_binary_operator(&context.names[&id], &arguments, context)),
        FunctionForm::FunctionCall(id) => Box::new(ast::Expression::NamedReference(context.names[id].clone())),
        FunctionForm::CallAsFunction => {
            parameters.remove(0);
            transpile_expression(arguments.remove(0), context)
        },
        FunctionForm::GetMemberField(id) => {
            assert_eq!(arguments.len(), 1);
            let object = transpile_expression(arguments[0], context);
            return Left(Box::new(ast::Expression::MemberAccess(object, context.names[id].clone())))
        },
        FunctionForm::SetMemberField(id) => {
            assert_eq!(arguments.len(), 2);
            return Right(Box::new(ast::Statement::VariableAssignment {
                target: Box::new(ast::Expression::MemberAccess(transpile_expression(arguments[0], context), context.names[id].clone())),
                value: Some(transpile_expression(arguments[1], context)),
                type_annotation: None,
            }))
        }
        FunctionForm::MemberCall(id) => {
            parameters.remove(0);
            let object = transpile_expression(arguments.remove(0), context);
            Box::new(ast::Expression::MemberAccess(object, context.names[id].clone()))
        },
    };

    for (parameter, argument) in zip_eq(parameters.iter(), arguments.iter()) {
        py_arguments.push((parameter.external_key.clone(), transpile_expression(argument.clone(), context)));
    }

    return Left(Box::new(ast::Expression::FunctionCall(target, py_arguments)))
}

pub fn try_transpile_optimization(function: &Rc<FunctionHead>, expression_id: &ExpressionID, arguments: &Vec<ExpressionID>, context: &FunctionContext) -> Option<Box<ast::Expression>> {
    guard!(let Some(descriptor) = context.logic.get(function) else {
        return None;
    });

    // TODO Monoteny should instead offer its own parser function, and we simply optimize calls that have python-parseable literals.
    Some(match descriptor {
        FunctionLogicDescriptor::PrimitiveOperation { type_, operation } => {
            match operation {
                PrimitiveOperation::ParseIntString => transpile_parse_function("^[0-9]+$", arguments, expression_id, context),
                PrimitiveOperation::ParseRealString => transpile_parse_function("^[0-9]+\\.[0-9]*$", arguments, expression_id, context),
                _ => return None,
            }
        }
        _ => return None,
    })
}

pub fn escape_string(string: &str) -> String {
    strings::map_chars(string, |ch| {
        Some(match ch {
            '\\' => "\\\\",
            '\n' => "\\n",
            '\0' => "\\0",
            '\t' => "\\t",
            '\r' => "\\r",
            '\"' => "\\\"",
            _ => return None,
        })
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

    let value = match &context.expressions.values[&argument_expression_id] {
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
