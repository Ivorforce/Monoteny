use std::collections::HashMap;
use std::rc::Rc;

use itertools::Either::{Left, Right};
use itertools::{zip_eq, Either, Itertools};
use regex;
use uuid::Uuid;

use crate::program::expression_tree::*;
use crate::program::functions::{FunctionHead, FunctionImplementation, FunctionLogicDescriptor, ParameterKey, PrimitiveOperation};
use crate::program::generics::TypeForest;
use crate::transpiler::python::representations::{FunctionForm, Representations};
use crate::transpiler::python::{ast, types};

pub struct FunctionContext<'a> {
    pub names: &'a HashMap<Uuid, String>,

    pub representations: &'a Representations,
    pub logic: &'a HashMap<Rc<FunctionHead>, FunctionLogicDescriptor>,

    pub expressions: &'a ExpressionTree,
    pub types: &'a TypeForest,
}

pub fn transpile_function(function_head: &Rc<FunctionHead>, implementation: &FunctionImplementation, context: &FunctionContext) -> Box<ast::Statement> {
    match &context.representations.function_forms[function_head] {
        FunctionForm::Identity => panic!(),
        FunctionForm::Constant(id) => {
            Box::new(ast::Statement::VariableAssignment {
                target: Box::new(ast::Expression::NamedReference(context.names[id].clone())),
                value: Some(transpile_expression(implementation.expression_tree.root, context)),
                type_annotation: Some(types::transpile(&implementation.interface.return_type, context)),
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
                type_: types::transpile(&implementation.type_forest.resolve_type(&parameter.type_).unwrap(), context),
            })
        }).collect(),
        return_type: match implementation.interface.return_type.unit.is_void() {
            true => None,
            false => Some(types::transpile(&implementation.type_forest.resolve_type(&implementation.interface.return_type).unwrap(), context))
        },
        block: Box::new(ast::Block { statements: vec![] }),
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

    syntax.block = transpile_as_block(implementation, context, &implementation.expression_tree.root, true);

    syntax
}

fn transpile_block(implementation: &FunctionImplementation, context: &FunctionContext, statements: &Vec<ExpressionID>) -> Box<ast::Block> {
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
                    type_annotation: Some(types::transpile(&implementation.type_forest.resolve_type(&variable.type_).unwrap(), context)),
                })
            }
            ExpressionOperation::Return => {
                let value = implementation.expression_tree.children[&statement].iter().exactly_one().ok();
                Box::new(ast::Statement::Return(value.map(|value| transpile_expression(*value, context))))
            }
            ExpressionOperation::FunctionCall(call) => {
                match transpile_function_call(context, &call.function, &context.representations.function_forms[&call.function], *statement) {
                    Left(e) => Box::new(ast::Statement::Expression(e)),
                    Right(s) => s,
                }
            }
            ExpressionOperation::IfThenElse => {
                // Build up elifs from nested if else { if } expressions
                let mut current_if = Some((
                    &implementation.expression_tree.values[statement],
                    statement
                ));
                let mut if_thens = vec![];

                while let Some((ExpressionOperation::IfThenElse, expression)) = current_if {
                    let children = &implementation.expression_tree.children[expression];
                    let condition = transpile_expression(children[0], context);
                    let consequent = transpile_as_block(implementation, context, &children[1], false);

                    if_thens.push((condition, consequent));
                    current_if = children.get(2).map(|a| (&implementation.expression_tree.values[a], a));
                };

                let alternative = current_if.map(|(_, a)| transpile_as_block(implementation, context, a, false));

                Box::new(ast::Statement::IfThenElse(if_thens, alternative))
            }
            _ => Box::new(ast::Statement::Expression(transpile_expression(*statement, context))),
        });
    }

    Box::new(ast::Block { statements: statements_ })
}

fn transpile_as_block(implementation: &FunctionImplementation, context: &FunctionContext, expression: &ExpressionID, auto_return: bool) -> Box<ast::Block> {
    match &implementation.expression_tree.values[expression] {
        ExpressionOperation::Block => {
            transpile_block(&implementation, context, &implementation.expression_tree.children[expression])
        }
        _ => {
            let expression = transpile_expression(*expression, context);

            Box::new(ast::Block { statements: vec![Box::new(match !auto_return && implementation.interface.return_type.unit.is_void() {
                true => ast::Statement::Expression(expression),
                false => ast::Statement::Return(Some(expression)),
            })] })
        }
    }
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
            let form = &context.representations.function_forms.get(&call.function).unwrap_or_else(|| panic!("Unable to get function form for {:?}", call.function));
            match transpile_function_call(context, &call.function, form, expression_id) {
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
        ExpressionOperation::IfThenElse => panic!("If-Then-Else not allowed as expression."),
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
        FunctionForm::Identity => return Left(transpile_expression(arguments[0], context)),
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
    let Some(descriptor) = context.logic.get(function) else {
        return None;
    };

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

pub fn transpile_unary_operator(operator: &str, arguments: &Vec<ExpressionID>, context: &FunctionContext) -> Box<ast::Expression> {
    let [expression] = arguments[..] else {
        panic!("Unary operator got {} arguments: {}", arguments.len(), operator);
    };

    Box::new(ast::Expression::UnaryOperation(operator.to_string(), transpile_expression(expression, context)))
}

pub fn transpile_binary_operator(operator: &str, arguments: &Vec<ExpressionID>, context: &FunctionContext) -> Box<ast::Expression> {
    let [lhs, rhs] = arguments[..] else {
        panic!("Binary operator got {} arguments: {}", arguments.len(), operator);
    };

    Box::new(ast::Expression::BinaryOperation(transpile_expression(lhs, context), operator.to_string(), transpile_expression(rhs, context)))
}

pub fn transpile_parse_function(supported_regex: &str, arguments: &Vec<ExpressionID>, expression_id: &ExpressionID, context: &FunctionContext) -> Box<ast::Expression> {
    let [argument_expression_id] = arguments[..] else {
        panic!("Parse function got {} arguments", arguments.len());
    };

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
