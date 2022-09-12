use std::rc::Rc;
use uuid::Uuid;
use std::iter::zip;
use crate::linker;
use crate::linker::computation_tree::{Expression, ExpressionOperation, Function, PassedArgument, Statement};
use crate::linker::scopes;
use crate::parser::abstract_syntax;
use crate::program::builtins::TenLangBuiltins;
use crate::program::primitives;
use crate::program::types::*;

pub struct ImperativeResolver<'a> {
    pub interface: &'a Rc<FunctionInterface>,
    pub builtins: &'a TenLangBuiltins,
}

impl <'a> ImperativeResolver<'a> {
    pub fn link_function_body(&self, body: &Vec<Box<abstract_syntax::Statement>>, scope: &scopes::Hierarchy) -> Rc<Function> {
        let mut parameter_variables = scopes::Level::new();

        for parameter in &self.interface.parameters {
            let variable = &parameter.variable;
            parameter_variables.insert_singleton(scopes::Environment::Global, variable.clone());
        }

        let subscope = scope.subscope(&parameter_variables);
        let statements: Vec<Box<Statement>> = self.link_top_scope(body, &subscope);

        return Rc::new(Function {
            interface: Rc::clone(self.interface),
            statements,
        });
    }

    pub fn link_top_scope(&self, body: &Vec<Box<abstract_syntax::Statement>>, scope: &scopes::Hierarchy) -> Vec<Box<Statement>> {
        if let Some(_) = &self.interface.return_type {
            if let [statement] = &body[..] {
                if let abstract_syntax::Statement::Expression(expression ) = statement.as_ref() {
                    // Single-Statement Return
                    return vec![Box::new(Statement::Return(Some(self.link_expression(expression, &scope))))]
                }
            }
        }

        self.link_scope(body, &scope)
    }

    pub fn link_scope(&self, body: &Vec<Box<abstract_syntax::Statement>>, scope: &scopes::Hierarchy) -> Vec<Box<Statement>> {
        let mut local_variables = scopes::Level::new();
        let mut statements: Vec<Box<Statement>> = Vec::new();

        for statement in body.iter() {
            match statement.as_ref() {
                abstract_syntax::Statement::VariableDeclaration {
                    mutability, identifier, type_declaration, expression
                } => {
                    let subscope = scope.subscope(&local_variables);
                    let expression: Box<Expression> = self.link_expression(&expression, &subscope);
                    let inferred_type = expression.result_type.as_ref().unwrap();

                    if let Some(type_declaration) = type_declaration {
                        let type_declaration = linker::link_type(&type_declaration, &subscope);
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
                    let expression: Option<Box<Expression>> = expression.as_ref().map(|x| self.link_expression(x, &subscope));

                    match (&self.interface.return_type, expression) {
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
                    let expression: Box<Expression> = self.link_expression(&expression, &subscope);
                    statements.push(Box::new(Statement::Expression(expression)));
                }
                abstract_syntax::Statement::VariableAssignment { variable_name, new_value } => {
                    let subscope = scope.subscope(&local_variables);
                    let variable = subscope.resolve_unambiguous(scopes::Environment::Global, variable_name);

                    if variable.mutability == Mutability::Immutable {
                        panic!("Cannot assign to immutable variable '{}'.", variable_name);
                    }

                    let new_value: Box<Expression> = self.link_expression(&new_value, &subscope);

                    statements.push(Box::new(
                        Statement::VariableAssignment(Rc::clone(&variable), new_value)
                    ));
                }
            }
        }

        statements
    }

    pub fn link_expression(&self, syntax: &abstract_syntax::Expression, scope: &scopes::Hierarchy) -> Box<Expression> {
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
                    result_type: Some(Box::new(Type::Struct(Rc::clone(&self.builtins.structs.String))))
                })
            },
            abstract_syntax::Expression::ArrayLiteral(raw_elements) => {
                let elements: Vec<Box<Expression>>= raw_elements.iter()
                    .map(|x| self.link_expression(x, scope))
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
                let lhs = self.link_expression(lhs, scope);
                let rhs = self.link_expression(rhs, scope);
                let function = link_binary_function(&lhs, operator, &rhs, scope);

                link_static_function_call(
                    function, link_arguments_to_parameters(function, vec![lhs, rhs])
                )
            },
            abstract_syntax::Expression::ConjunctivePairOperators { arguments, operators } => {
                let arguments: Vec<Box<Expression>> = arguments.into_iter()
                    .map(|x| self.link_expression(x, scope))
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
                let argument = self.link_expression(argument, scope);
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
                        let arguments: Vec<Box<PassedArgument>> = self.link_passed_arguments(
                            arguments.iter(), scope, 0
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
                                let target = self.link_expression(target, scope);

                                // Member Function
                                let arguments: Vec<Box<PassedArgument>> = Some(Box::new(PassedArgument {
                                    key: ParameterKey::Int(0),
                                    value: target
                                })).into_iter().chain(self.link_passed_arguments(
                                    arguments.iter(), scope, 1
                                ).into_iter()).collect();

                                let argument_types = arguments.iter()
                                    .map(|x| x.to_argument_type())
                                    .collect();

                                let function = scope.resolve_function(scopes::Environment::Member, member_name, &argument_types);

                                link_static_function_call(function, arguments)
                            },
                            _ => {
                                // Higher order function
                                let target = self.link_expression(callee, scope);

                                let arguments: Vec<Box<PassedArgument>> = self.link_passed_arguments(arguments.iter(), scope, 0);

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

    pub fn link_passed_arguments<'b, I>(&self, arguments: I, scope: &scopes::Hierarchy, offset: usize) -> Vec<Box<PassedArgument>> where I: Iterator<Item = &'b Box<abstract_syntax::PassedArgument>> {
        arguments.enumerate()
            .map(|(idx, x)| {
                Box::new(PassedArgument {
                    key: link_parameter_key_option(&x.key, idx + offset),
                    value: self.link_expression(&x.value, scope)
                })
            })
            .collect()
    }
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

pub fn link_parameter_key_option(key: &Option<ParameterKey>, index: usize) -> ParameterKey {
    if let Some(key) = key {
        linker::link_parameter_key(key, index)
    }
    else {
        ParameterKey::Int(index as i32)
    }
}
