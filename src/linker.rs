pub mod builtins;
pub mod computation_tree;
pub mod scopes;
pub mod primitives;

use std::collections::HashMap;
use std::iter::zip;
use std::rc::Rc;
use guard::guard;
use uuid::Uuid;

use crate::abstract_syntax;
use crate::linker::builtins::*;
use crate::linker::computation_tree::*;
use crate::linker::scopes::Environment;


pub fn link_program(syntax: abstract_syntax::Program) -> Program {
    let builtins = create_builtins();
    let builtin_variable_scope = builtins.global_constants.as_global_scope();

    let mut functions_with_bodies: Vec<(Rc<FunctionInterface>, &Vec<Box<abstract_syntax::Statement>>)> = Vec::new();
    let mut global_variables = scopes::Level::new();

    // Resolve things in global scope
    for statement in &syntax.global_statements {
        match statement.as_ref() {
            abstract_syntax::GlobalStatement::FunctionDeclaration(function) => {
                let interface = link_function_interface(&function, &builtin_variable_scope);

                functions_with_bodies.push((Rc::clone(&interface), &function.body));

                let environment = match interface.is_member_function {
                    true => scopes::Environment::Global,
                    false => scopes::Environment::Member,
                };

                global_variables.add_function(environment, Rc::new(Variable {
                    id: Uuid::new_v4(),
                    name: interface.name.clone(),
                    type_declaration: Box::new(Type::Function(Rc::clone(&interface))),
                    mutability: abstract_syntax::Mutability::Immutable,
                }));

                if !interface.is_member_function {
                    // Create a variable for the function
                }
                else {
                    // TODO Otherwise, create a variable as Metatype.function.
                }
            }
        }
    }

    let global_variable_scope = builtin_variable_scope.subscope(&global_variables);

    // Resolve function bodies
    let functions: Vec<Rc<Function>> = functions_with_bodies.into_iter().map(
        |(interface, statements)|
        link_function_body(statements, &interface, &global_variable_scope, &builtins)
    ).collect();

    return Program {
        functions,
        builtins
    }
}

pub fn link_function_interface(function: &abstract_syntax::Function, scope: &scopes::Hierarchy) -> Rc<FunctionInterface> {
    let return_type = function.return_type.as_ref().map(|x| link_type(&x, scope));

    let mut parameters: Vec<Box<Parameter>> = Vec::new();

    if let Some(target_type) = &function.target_type {
        let variable = Rc::new(Variable {
            id: Uuid::new_v4(),
            name: String::from("self"),
            type_declaration: link_type(&target_type, scope),
            mutability: abstract_syntax::Mutability::Immutable,
        });

        parameters.push(Box::new(Parameter {
            external_key: ParameterKey::Int(0),
            variable
        }));
    }

    for parameter in function.parameters.iter() {
        let variable = Rc::new(Variable {
            id: Uuid::new_v4(),
            name: parameter.internal_name.clone(),
            type_declaration: link_type(parameter.param_type.as_ref(), scope),
            mutability: abstract_syntax::Mutability::Immutable,
        });

        parameters.push(Box::new(Parameter {
            external_key: link_parameter_key(&parameter.key, parameters.len()),
            variable
        }));
    }

    return Rc::new(FunctionInterface {
        id: Uuid::new_v4(),
        name: function.identifier.clone(),

        is_member_function: function.target_type.is_none(),
        parameters,
        // This is correct so far because syntax does not allow generics use yet.
        generics: vec![],

        return_type
    });
}

pub fn link_passed_arguments<'a, I>(arguments: I, scope: &scopes::Hierarchy, builtins: &TenLangBuiltins, offset: usize) -> Vec<Box<PassedArgument>> where I: Iterator<Item = &'a Box<abstract_syntax::PassedArgument>> {
    arguments.enumerate()
        .map(|(idx, x)| {
            Box::new(PassedArgument {
                key: link_parameter_key_option(&x.key, idx + offset),
                value: link_expression(&x.value, scope, builtins)
            })
        })
        .collect()
}

pub fn link_parameter_key(key: &abstract_syntax::ParameterKey, index: usize) -> ParameterKey {
    match key {
        abstract_syntax::ParameterKey::Int(n) => ParameterKey::Int(*n),
        abstract_syntax::ParameterKey::Name(n) => {
            match n.as_str() {
                // When _ a: SomeType is declared, it is keyed by its index.
                "_" => ParameterKey::Int(index as i32),
                _ => ParameterKey::Name(n.clone())
            }
        },
    }
}

pub fn link_parameter_key_option(key: &Option<abstract_syntax::ParameterKey>, index: usize) -> ParameterKey {
    if let Some(key) = key {
        link_parameter_key(key, index)
    }
    else {
        ParameterKey::Int(index as i32)
    }
}

pub fn link_function_body(body: &Vec<Box<abstract_syntax::Statement>>, interface: &Rc<FunctionInterface>, scope: &scopes::Hierarchy, builtins: &TenLangBuiltins) -> Rc<Function> {
    let mut parameter_variables = scopes::Level::new();

    for parameter in &interface.parameters {
        let variable = &parameter.variable;
        parameter_variables.insert_singleton(Environment::Global, variable.clone());
    }

    let subscope = scope.subscope(&parameter_variables);
    let statements: Vec<Box<Statement>> = link_top_scope(body, interface, &subscope, builtins);

    return Rc::new(Function {
        interface: Rc::clone(interface),
        statements,
    });
}

pub fn link_top_scope(body: &Vec<Box<abstract_syntax::Statement>>, interface: &Rc<FunctionInterface>, scope: &scopes::Hierarchy, builtins: &TenLangBuiltins) -> Vec<Box<Statement>> {
    if let Some(_) = &interface.return_type {
        if let [statement] = &body[..] {
            if let abstract_syntax::Statement::Expression(expression ) = statement.as_ref() {
                // Single-Statement Return
                return vec![Box::new(Statement::Return(Some(link_expression(expression, &scope, builtins))))]
            }
        }
    }

    link_scope(body, &interface, &scope, builtins)
}

pub fn link_scope(body: &Vec<Box<abstract_syntax::Statement>>, interface: &Rc<FunctionInterface>, scope: &scopes::Hierarchy, builtins: &TenLangBuiltins) -> Vec<Box<Statement>> {
    let mut local_variables = scopes::Level::new();
    let mut statements: Vec<Box<Statement>> = Vec::new();

    for statement in body.iter() {
        match statement.as_ref() {
            abstract_syntax::Statement::VariableDeclaration {
                mutability, identifier, type_declaration, expression
            } => {
                let subscope = scope.subscope(&local_variables);
                let expression: Box<Expression> = link_expression(&expression, &subscope, builtins);
                let inferred_type = expression.result_type.as_ref().unwrap();

                if let Some(type_declaration) = type_declaration {
                    let type_declaration = link_type(&type_declaration, &subscope);
                    if &type_declaration != inferred_type {
                        panic!("Declared type of variable '{}' is not equal to inferred type '{:?}'", identifier, inferred_type);
                    }
                }

                let variable = Rc::new(Variable {
                    id: Uuid::new_v4(),
                    name: identifier.clone(),
                    type_declaration: inferred_type.clone(),
                    mutability: mutability.clone(),
                });

                statements.push(Box::new(
                    Statement::VariableAssignment(Rc::clone(&variable), expression)
                ));
                local_variables.push_variable(Environment::Global, variable);
            },
            abstract_syntax::Statement::Return(expression) => {
                let subscope = scope.subscope(&local_variables);
                let expression: Option<Box<Expression>> = expression.as_ref().map(|x| link_expression(x, &subscope, builtins));

                match (&interface.return_type, expression) {
                    (Some(_), None) => panic!("Return statement offers no value when the function declares an object."),
                    (None, Some(_)) => panic!("Return statement offers a value when the function declares void."),
                    (None, None) => {
                        statements.push(Box::new(Statement::Return(None)));
                    },
                    (Some(interface_return_type), Some(expression)) => {
                        match &expression.result_type {
                            None => panic!("Return statement expression resolves to void. Please move the expression into a separate line."),
                            Some(result_type) => {
                                if interface_return_type != result_type {
                                    panic!("Return statement offers incompatible value of type: '{:?}' when function declares type: '{:?}'", result_type, interface_return_type);
                                }

                                statements.push(Box::new(Statement::Return(Some(expression))));
                            }
                        }
                    }
                }
            },
            abstract_syntax::Statement::Expression(expression) => {
                let subscope = scope.subscope(&local_variables);
                let expression: Box<Expression> = link_expression(&expression, &subscope, builtins);
                statements.push(Box::new(Statement::Expression(expression)));
            }
            abstract_syntax::Statement::VariableAssignment(name, expression) => {
                let subscope = scope.subscope(&local_variables);
                let variable = subscope.resolve_unambiguous(Environment::Global, name);

                if variable.mutability == abstract_syntax::Mutability::Immutable {
                    panic!("Cannot assign to immutable variable '{}'.", name);
                }

                let expression: Box<Expression> = link_expression(&expression, &subscope, builtins);

                statements.push(Box::new(
                    Statement::VariableAssignment(Rc::clone(&variable), expression)
                ));
            }
        }
    }

    statements
}

pub fn link_binary_function<'a>(lhs: &Expression, operator: &'a abstract_syntax::BinaryOperator, rhs: &Expression, scope: &'a scopes::Hierarchy) -> &'a Rc<FunctionInterface> {
    let call_arguments = vec![
        PassedArgumentType { key: ParameterKey::Int(0), value: &lhs.result_type },
        PassedArgumentType { key: ParameterKey::Int(1), value: &rhs.result_type },
    ];
    scope.resolve_function(Environment::Global, &format!("{:?}", operator), &call_arguments)
}

pub fn link_unary_function<'a>(operator: &'a abstract_syntax::UnaryOperator, value: &Expression, scope: &'a scopes::Hierarchy) -> &'a Rc<FunctionInterface> {
    let call_arguments = vec![
        PassedArgumentType { key: ParameterKey::Int(0), value: &value.result_type },
    ];
    scope.resolve_function(Environment::Global, &format!("{:?}", operator), &call_arguments)
}

fn link_static_function_call(function: &Rc<FunctionInterface>, arguments: Vec<Box<PassedArgument>>) -> Box<Expression> {
    Box::new(Expression {
        result_type: function.return_type.clone(),
        operation: Box::new(ExpressionOperation::StaticFunctionCall {
            function: Rc::clone(function),
            arguments
        })
    })
}

pub fn link_arguments_to_parameters(function: &Rc<FunctionInterface>, arguments: Vec<Box<Expression>>) -> Vec<Box<PassedArgument>> {
    arguments.into_iter()
        .enumerate()
        .map(|(idx, argument)| Box::new(PassedArgument { key: function.parameters[idx].external_key.clone(), value: argument }))
        .collect()
}

pub fn link_expression(syntax: &abstract_syntax::Expression, scope: &scopes::Hierarchy, builtins: &TenLangBuiltins) -> Box<Expression> {
    match syntax {
        abstract_syntax::Expression::Number(n) => {
            // TODO The type should be inferred
            let value = primitives::Value::Int32(n.clone());
            Box::new(Expression {
                result_type: Some(Box::new(Type::Primitive(value.get_type()))),
                operation: Box::new(ExpressionOperation::Primitive(value)),
            })
        },
        abstract_syntax::Expression::Bool(n) => Box::new(Expression {
            operation: Box::new(ExpressionOperation::Primitive(primitives::Value::Bool(n.clone()))),
            result_type: Some(Box::new(Type::Primitive(primitives::Type::Bool)))
        }),
        abstract_syntax::Expression::StringLiteral(string) => {
            Box::new(Expression {
                operation: Box::new(ExpressionOperation::StringLiteral(string.clone())),
                result_type: Some(Box::new(Type::Struct(Rc::clone(&builtins.structs.String))))
            })
        },
        abstract_syntax::Expression::ArrayLiteral(raw_elements) => {
            let elements: Vec<Box<Expression>>= raw_elements.iter()
                .map(|x| link_expression(x, scope, builtins))
                .collect();

            let supertype = get_common_supertype(
                &elements.iter().map(|x| x.result_type.as_ref().unwrap()).collect()
            ).clone();

            Box::new(Expression {
                operation: Box::new(ExpressionOperation::ArrayLiteral(elements)),
                result_type: Some(supertype)
            })
        },
        abstract_syntax::Expression::BinaryOperator { lhs, operator, rhs } => {
            let lhs = link_expression(lhs, scope, builtins);
            let rhs = link_expression(rhs, scope, builtins);
            let function = link_binary_function(&lhs, operator, &rhs, scope);

            link_static_function_call(
                function, link_arguments_to_parameters(function, vec![lhs, rhs])
            )
        },
        abstract_syntax::Expression::PairAssociativeBinaryOperator { arguments, operators } => {
            let arguments: Vec<Box<Expression>> = arguments.into_iter()
                .map(|x| link_expression(x, scope, builtins))
                .collect();

            let functions: Vec<Rc<FunctionInterface>> = zip(arguments.windows(2), operators.into_iter())
                .map(|(args, operator)| {
                    let (lhs, rhs) = (&args[0], &args[1]);
                    link_binary_function(lhs, operator, rhs, scope).clone()
                })
                .collect();

            if arguments.len() != functions.len() + 1 || arguments.len() < 2 {
                panic!("Internal comparison paris error (args.len(): {}, functions.len(): {})", arguments.len(), functions.len());
            }
            else if functions.len() == 1 {
                let function = &functions[0];

                // Just one pair, this is easy
                link_static_function_call(
                    function, link_arguments_to_parameters(function, arguments)
                )
            }
            else {
                Box::new(Expression {
                    // TODO This is not true; we have to see what (a > b) && (b > c) actually outputs
                    result_type: Some(Box::new(Type::Primitive(primitives::Type::Bool))),
                    operation: Box::new(ExpressionOperation::PairwiseOperations { arguments, functions })
                })
            }
        }
        abstract_syntax::Expression::UnaryOperator(operator, expression) => {
            let expression = link_expression(expression, scope, builtins);
            let function = link_unary_function(operator, &expression, scope);

            link_static_function_call(function, link_arguments_to_parameters(function, vec![expression]))
        },
        abstract_syntax::Expression::VariableLookup(identifier) => {
            let variable = scope.resolve_unambiguous(Environment::Global, identifier);

            Box::new(Expression {
                operation: Box::new(ExpressionOperation::VariableLookup(variable.clone())),
                result_type: Some(variable.type_declaration.clone())
            })
        },
        abstract_syntax::Expression::FunctionCall(call_type, callee, arguments) => {
            if call_type == &abstract_syntax::FunctionCallType::Subscript {
                panic!("Subscript not supported yet");
            }

            return match callee.as_ref() {
                abstract_syntax::Expression::VariableLookup(function_name) => {
                    // Static Call
                    let arguments: Vec<Box<PassedArgument>> = link_passed_arguments(
                        arguments.iter(), scope, builtins, 0
                    );

                    let argument_types = arguments.iter()
                        .map(|x| x.to_argument_type())
                        .collect();

                    let function = scope.resolve_function(Environment::Global, function_name, &argument_types);

                    link_static_function_call(function, arguments)
                }
                _ => {
                    match callee.as_ref() {
                        abstract_syntax::Expression::MemberLookup(target_object, member_name) => {
                            let target_obj = link_expression(target_object, scope, builtins);

                            // Member Function
                            let arguments: Vec<Box<PassedArgument>> = Some(Box::new(PassedArgument {
                                key: ParameterKey::Int(0),
                                value: target_obj
                            })).into_iter().chain(link_passed_arguments(
                                arguments.iter(), scope, builtins, 1
                            ).into_iter()).collect();

                            let argument_types = arguments.iter()
                                .map(|x| x.to_argument_type())
                                .collect();

                            let function = scope.resolve_function(Environment::Member, member_name, &argument_types);

                            link_static_function_call(function, arguments)
                        },
                        _ => {
                            // Higher order function
                            let target = link_expression(callee, scope, builtins);

                            let arguments: Vec<Box<PassedArgument>> = link_passed_arguments(arguments.iter(), scope, builtins, 0);

                            let function = match &target.result_type {
                                Some(result_type) => {
                                    match result_type.as_ref() {
                                        Type::Function(function) => function,
                                        _ => panic!("Expression does not resolve to a function."),
                                    }
                                }
                                _ => panic!("Expression does not return anything."),
                            };

                            link_static_function_call(function, arguments)
                        }
                    }
                }
            }
        }
        abstract_syntax::Expression::MemberLookup(_, _) => {
            todo!()
        }
    }
}

pub fn link_type(syntax: &abstract_syntax::TypeDeclaration, scope: &scopes::Hierarchy) -> Box<Type> {
    match syntax {
        abstract_syntax::TypeDeclaration::Identifier(id) => {
            scope.resolve_metatype(Environment::Global, id).clone()
        },
        abstract_syntax::TypeDeclaration::NDArray(identifier, _) => {
            Box::new(Type::NDArray(link_type(&identifier, scope)))
        }
    }
}
