pub mod computation_tree;
pub mod scopes;

use std::collections::HashMap;
use std::iter::zip;
use std::rc::Rc;
use guard::guard;
use uuid::Uuid;

use crate::parser;
use crate::parser::abstract_syntax;
use crate::linker::computation_tree::*;
use crate::program::primitives;
use crate::program::builtins::*;
use crate::program::types::*;


pub fn link_program(syntax: abstract_syntax::Program, parser_scope: &parser::scopes::Level, scope: &scopes::Hierarchy, builtins: &TenLangBuiltins) -> Program {
    let mut functions_with_bodies: Vec<(Rc<FunctionInterface>, &Vec<Box<abstract_syntax::Statement>>)> = Vec::new();
    let mut global_variables = scopes::Level::new();

    // Resolve things in global scope
    for statement in &syntax.global_statements {
        match statement.as_ref() {
            abstract_syntax::GlobalStatement::FunctionDeclaration(function) => {
                let interface = link_function_interface(&function, &scope);

                functions_with_bodies.push((Rc::clone(&interface), &function.body));

                let environment = match interface.form {
                    FunctionForm::Member => scopes::Environment::Member,
                    _ => scopes::Environment::Global,
                };

                // Create a variable for the function
                global_variables.add_function(environment, Rc::new(Variable {
                    id: Uuid::new_v4(),
                    name: interface.name.clone(),
                    type_declaration: Box::new(Type::Function(Rc::clone(&interface))),
                    mutability: Mutability::Immutable,
                }));

                // if interface.is_member_function {
                // TODO Create an additional variable as Metatype.function...?
                // }
            }
            abstract_syntax::GlobalStatement::Operator(operator) => {
                let interface = link_operator_interface(&operator, parser_scope, &scope);

                functions_with_bodies.push((Rc::clone(&interface), &operator.body));

                // Create a variable for the function
                global_variables.add_function(scopes::Environment::Global, Rc::new(Variable {
                    id: Uuid::new_v4(),
                    name: interface.name.clone(),
                    type_declaration: Box::new(Type::Function(Rc::clone(&interface))),
                    mutability: Mutability::Immutable,
                }));
            }
            _ => {}
        }
    }

    let global_variable_scope = scope.subscope(&global_variables);

    // Resolve function bodies
    let functions: Vec<Rc<Function>> = functions_with_bodies.into_iter().map(
        |(interface, statements)|
        link_function_body(statements, &interface, &global_variable_scope, &builtins)
    ).collect();

    return Program {
        functions,
    }
}

pub fn link_function_interface(function: &abstract_syntax::Function, scope: &scopes::Hierarchy) -> Rc<FunctionInterface> {
    let return_type = function.return_type.as_ref().map(|x| link_type(&x, scope));

    let mut parameters: Vec<Box<NamedParameter>> = Vec::new();

    if let Some(target) = &function.target {
        parameters.push(Box::new(NamedParameter {
            external_key: ParameterKey::Int(0),
            variable: link_contextual_parameter_as_variable(target, scope)
        }));
    }

    for parameter in function.parameters.iter() {
        let variable = Rc::new(Variable {
            id: Uuid::new_v4(),
            name: parameter.internal_name.clone(),
            type_declaration: link_type(parameter.param_type.as_ref(), scope),
            mutability: Mutability::Immutable,
        });

        parameters.push(Box::new(NamedParameter {
            external_key: link_parameter_key(&parameter.key, parameters.len()),
            variable
        }));
    }

    return Rc::new(FunctionInterface {
        id: Uuid::new_v4(),
        name: function.identifier.clone(),
        alphanumeric_name: function.identifier.clone(),

        form: if function.target.is_none() { FunctionForm::Global } else { FunctionForm::Member },
        parameters,
        // This is correct so far because syntax does not allow generics use yet.
        generics: vec![],

        return_type
    });
}

pub fn link_operator_interface(function: &abstract_syntax::Operator, parser_scope: &parser::scopes::Level, scope: &scopes::Hierarchy) -> Rc<FunctionInterface> {
    let return_type = function.return_type.as_ref().map(|x| link_type(&x, scope));

    let parameters: Vec<Box<NamedParameter>> = function.lhs.iter().chain([&function.rhs]).enumerate()
        .map(|(idx, x)| {
            Box::new(NamedParameter {
                external_key: ParameterKey::Int(idx as i32),
                variable: link_contextual_parameter_as_variable(x, scope)
            })
        })
        .collect();

    let is_binary = function.lhs.is_some();
    let pattern = parser_scope.resolve_operator_pattern(&function.operator, is_binary);

    return Rc::new(FunctionInterface {
        id: Uuid::new_v4(),
        name: function.operator.clone(),
        alphanumeric_name: pattern.alias.clone(),

        form: FunctionForm::Operator,
        parameters,
        // This is correct so far because syntax does not allow generics use yet.
        generics: vec![],

        return_type
    });
}

pub fn link_contextual_parameter_as_variable(parameter: &abstract_syntax::ContextualParameter, scope: &scopes::Hierarchy) -> Rc<Variable> {
    Rc::new(Variable {
        id: Uuid::new_v4(),
        name: String::from(&parameter.internal_name),
        type_declaration: link_type(&parameter.param_type, scope),
        mutability: Mutability::Immutable,
    })
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

pub fn link_parameter_key(key: &ParameterKey, index: usize) -> ParameterKey {
    match key {
        ParameterKey::Int(n) => ParameterKey::Int(*n),
        ParameterKey::Name(n) => {
            match n.as_str() {
                // When _ a: SomeType is declared, it is keyed by its index.
                "_" => ParameterKey::Int(index as i32),
                _ => ParameterKey::Name(n.clone())
            }
        },
    }
}

pub fn link_parameter_key_option(key: &Option<ParameterKey>, index: usize) -> ParameterKey {
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
        parameter_variables.insert_singleton(scopes::Environment::Global, variable.clone());
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
                local_variables.push_variable(scopes::Environment::Global, variable);
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
            abstract_syntax::Statement::VariableAssignment { variable_name, new_value } => {
                let subscope = scope.subscope(&local_variables);
                let variable = subscope.resolve_unambiguous(scopes::Environment::Global, variable_name);

                if variable.mutability == Mutability::Immutable {
                    panic!("Cannot assign to immutable variable '{}'.", variable_name);
                }

                let new_value: Box<Expression> = link_expression(&new_value, &subscope, builtins);

                statements.push(Box::new(
                    Statement::VariableAssignment(Rc::clone(&variable), new_value)
                ));
            }
        }
    }

    statements
}

pub fn link_binary_function<'a>(lhs: &Expression, operator: &'a String, rhs: &Expression, scope: &'a scopes::Hierarchy) -> &'a Rc<FunctionInterface> {
    let call_arguments = vec![
        PassedArgumentType { key: ParameterKey::Int(0), value: &lhs.result_type },
        PassedArgumentType { key: ParameterKey::Int(1), value: &rhs.result_type },
    ];
    scope.resolve_function(scopes::Environment::Global, operator, &call_arguments)
}

pub fn link_unary_function<'a>(operator: &'a String, value: &Expression, scope: &'a scopes::Hierarchy) -> &'a Rc<FunctionInterface> {
    let call_arguments = vec![
        PassedArgumentType { key: ParameterKey::Int(0), value: &value.result_type },
    ];
    scope.resolve_function(scopes::Environment::Global, operator, &call_arguments)
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
        abstract_syntax::Expression::ConjunctivePairOperators { arguments, operators } => {
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
                panic!("Internal Error for PairAssociativeBinaryOperators: (args.len(): {}, functions.len(): {})", arguments.len(), functions.len());
            }
            else {
                if functions.len() == 1 {
                    println!("Warning: Attempting making a pair-associative operator from just 1 pair. This should not happen.");
                }

                Box::new(Expression {
                    // TODO This is not true; we have to see what (a > b) && (b > c) actually outputs
                    result_type: Some(Box::new(Type::Primitive(primitives::Type::Bool))),
                    operation: Box::new(ExpressionOperation::PairwiseOperations { arguments, functions })
                })
            }
        }
        abstract_syntax::Expression::UnaryOperator { operator, argument} => {
            let argument = link_expression(argument, scope, builtins);
            let function = link_unary_function(operator, &argument, scope);

            link_static_function_call(function, link_arguments_to_parameters(function, vec![argument]))
        },
        abstract_syntax::Expression::VariableLookup(identifier) => {
            let variable = scope.resolve_unambiguous(scopes::Environment::Global, identifier);

            Box::new(Expression {
                operation: Box::new(ExpressionOperation::VariableLookup(variable.clone())),
                result_type: Some(variable.type_declaration.clone())
            })
        },
        abstract_syntax::Expression::FunctionCall { call_type, callee, arguments } => {
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

                    let function = scope.resolve_function(scopes::Environment::Global, function_name, &argument_types);

                    link_static_function_call(function, arguments)
                }
                _ => {
                    match callee.as_ref() {
                        abstract_syntax::Expression::MemberLookup { target, member_name } => {
                            let target = link_expression(target, scope, builtins);

                            // Member Function
                            let arguments: Vec<Box<PassedArgument>> = Some(Box::new(PassedArgument {
                                key: ParameterKey::Int(0),
                                value: target
                            })).into_iter().chain(link_passed_arguments(
                                arguments.iter(), scope, builtins, 1
                            ).into_iter()).collect();

                            let argument_types = arguments.iter()
                                .map(|x| x.to_argument_type())
                                .collect();

                            let function = scope.resolve_function(scopes::Environment::Member, member_name, &argument_types);

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
        abstract_syntax::Expression::MemberLookup { target, member_name } => {
            todo!()
        }
        abstract_syntax::Expression::UnsortedBinaryOperators { .. } => {
            panic!("Internal Error: Unsorted binary operators should not occur at this stage.")
        }
    }
}

pub fn link_type(syntax: &abstract_syntax::TypeDeclaration, scope: &scopes::Hierarchy) -> Box<Type> {
    match syntax {
        abstract_syntax::TypeDeclaration::Identifier(id) => {
            scope.resolve_metatype(scopes::Environment::Global, id).clone()
        },
        abstract_syntax::TypeDeclaration::Monad { unit, shape } => {
            Box::new(Type::Monad(link_type(&unit, scope)))
        }
    }
}
