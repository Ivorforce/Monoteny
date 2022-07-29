pub mod builtins;
pub mod computation_tree;
pub mod scopes;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::rc::Rc;
use guard::guard;
use uuid::Uuid;

use crate::abstract_syntax;
use crate::linker::builtins::*;
use crate::linker::computation_tree::*;
use crate::linker::scopes::*;


pub fn resolve_program(syntax: abstract_syntax::Program) -> Program {
    let (builtins, builtins_variables) = create_builtins();
    let builtin_variable_scope = ScopedVariables {
        scopes: vec![&builtins_variables],
    };

    let mut functions_with_bodies: Vec<(Rc<FunctionInterface>, &Vec<Box<abstract_syntax::Statement>>)> = Vec::new();
    let mut global_variables: HashMap<String, Rc<Variable>> = HashMap::new();

    // Resolve things in global scope
    for statement in &syntax.global_statements {
        match statement.as_ref() {
            abstract_syntax::GlobalStatement::FunctionDeclaration(function) => {
                let interface = resolve_function_interface(&function);

                if global_variables.contains_key(&interface.name) {
                    panic!("Duplicate definition for global '{}'", interface.name);
                }

                functions_with_bodies.push((Rc::clone(&interface), &function.body));

                // Create a variable for the function
                global_variables.insert(interface.name.clone(), Rc::new(Variable {
                    id: Uuid::new_v4(),
                    name: interface.name.clone(),
                    type_declaration: Box::new(Type::Function(Rc::clone(&interface))),
                    mutability: abstract_syntax::Mutability::Immutable,
                }));
            }
            abstract_syntax::GlobalStatement::Extension(_) => {
                todo!()
            }
        }
    }

    let global_variable_scope = builtin_variable_scope.subscope(&global_variables);

    // Resolve function bodies
    let functions: Vec<Rc<Function>> = functions_with_bodies.into_iter().map(
        |(interface, statements)|
        resolve_function_body(statements, interface, &global_variable_scope, &builtins)
    ).collect();

    return Program {
        variables: HashMap::new(),
        functions,
        builtins
    }
}

pub fn resolve_function_interface(function: &abstract_syntax::Function) -> Rc<FunctionInterface> {
    let return_type = function.return_type.as_ref().map(|x| resolve_type(&x));

    let mut parameters: Vec<Box<Parameter>> = Vec::new();

    for (idx, parameter) in function.parameters.iter().enumerate() {
        let variable = Rc::new(Variable {
            id: Uuid::new_v4(),
            name: parameter.internal_name.clone(),
            type_declaration: resolve_type(parameter.param_type.as_ref()),
            mutability: abstract_syntax::Mutability::Immutable,
        });

        parameters.push(Box::new(Parameter {
            external_key: resolve_parameter_key(&parameter.key, idx),
            variable
        }));
    }

    return Rc::new(FunctionInterface {
        id: Uuid::new_v4(),
        name: function.identifier.clone(),

        parameters,
        // This is correct so far because syntax does not allow generics use yet.
        generics: vec![],

        return_type
    });
}

pub fn resolve_parameter_key(key: &abstract_syntax::ParameterKey, index: usize) -> ParameterKey {
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

pub fn resolve_function_body(body: &Vec<Box<abstract_syntax::Statement>>, interface: Rc<FunctionInterface>, global_variables: &ScopedVariables, builtins: &TenLangBuiltins) -> Rc<Function> {
    let mut local_variables: HashMap<String, Rc<Variable>> = HashMap::new();

    for parameter in &interface.parameters {
        let variable = &parameter.variable;
        local_variables.insert(variable.name.clone(), variable.clone());
    }

    let mut statements: Vec<Box<Statement>> = Vec::new();

    for statement in body.iter() {
        match statement.as_ref() {
            abstract_syntax::Statement::VariableDeclaration {
                mutability, identifier, type_declaration, expression
            } => {
                let variables = global_variables.subscope(&local_variables);
                let expression: Box<Expression> = resolve_expression(&expression, &variables, builtins);
                let inferred_type = expression.result_type.as_ref().unwrap();

                if let Some(type_declaration) = type_declaration {
                    let type_declaration = resolve_type(&type_declaration);
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
                local_variables.insert(identifier.clone(), variable);
            },
            abstract_syntax::Statement::Return(None) => {
                statements.push(Box::new(Statement::Return(None)));
            },
            abstract_syntax::Statement::Return(Some(expression)) => {
                let variables = global_variables.subscope(&local_variables);
                let expression: Box<Expression> = resolve_expression(&expression, &variables, builtins);

                if interface.return_type != expression.result_type {
                    panic!("Declared type of return statement is not equal to function return type '{:?}'", expression.result_type);
                }

                statements.push(Box::new(Statement::Return(Some(expression))));
            },
            abstract_syntax::Statement::Expression(expression) => {
                let variables = global_variables.subscope(&local_variables);
                let expression: Box<Expression> = resolve_expression(&expression, &variables, builtins);
                statements.push(Box::new(Statement::Expression(expression)));
            }
            abstract_syntax::Statement::VariableAssignment(name, expression) => {
                let variables = global_variables.subscope(&local_variables);
                let variable = variables.resolve(name);

                if variable.mutability == abstract_syntax::Mutability::Immutable {
                    panic!("Cannot assign to immutable variable '{}'.", name);
                }

                let expression: Box<Expression> = resolve_expression(&expression, &variables, builtins);

                statements.push(Box::new(
                    Statement::VariableAssignment(Rc::clone(&variable), expression)
                ));
            }
        }
    }

    return Rc::new(Function {
        interface,
        variables: local_variables.values().map(|variable| (variable.id, variable.clone())).collect(),
        statements,
    });
}

pub fn resolve_static_function_call(function: &Rc<FunctionInterface>, arguments: &Vec<&Box<abstract_syntax::Expression>>, variables: &ScopedVariables, builtins: &TenLangBuiltins) -> Expression {
    let resolved_arguments: Vec<Box<Expression>> = arguments.into_iter()
        .map(|x| resolve_expression(x, variables, builtins))
        .collect();

    guard!(let Some(reference) = resolved_arguments.first() else {
        // TODO
        panic!("operators without arguments are not supported yet - because generic return types are not supported yet.")
    });

    for other in resolved_arguments.iter().skip(1) {
        if &other.result_type.clone() != &reference.result_type.clone() {
            // TODO
            panic!("operator sides must be of the same result type - because generic return types are not supported yet.")
        }
    }

    let result_type = reference.result_type.clone();

    Expression {
        operation: Box::new(ExpressionOperation::StaticFunctionCall {
            arguments: resolved_arguments.into_iter()
                .enumerate()
                .map(|(idx, argument)| Box::new(PassedArgument { key: function.parameters[idx].external_key.clone(), value: argument }))
                .collect(),
            function: function.clone(),
        }),
        result_type
    }
}

pub fn resolve_expression(syntax: &abstract_syntax::Expression, variables: &ScopedVariables, builtins: &TenLangBuiltins) -> Box<Expression> {
    Box::new(match syntax {
        abstract_syntax::Expression::Number(n) => {
            // TODO The type should be inferred
            let primitive = Primitive::Int32(n.clone());
            Expression {
                result_type: Some(Box::new(Type::Primitive(primitive.primitive_type()))),
                operation: Box::new(ExpressionOperation::Primitive(primitive)),
            }
        },
        abstract_syntax::Expression::Bool(n) => Expression {
            operation: Box::new(ExpressionOperation::Primitive(Primitive::Bool(n.clone()))),
            result_type: Some(Box::new(Type::Primitive(PrimitiveType::Bool)))
        },
        abstract_syntax::Expression::StringLiteral(string) => {
            Expression {
                operation: Box::new(ExpressionOperation::StringLiteral(string.clone())),
                result_type: Some(Box::new(Type::Identifier(String::from("String"))))
            }
        },
        abstract_syntax::Expression::ArrayLiteral(raw_elements) => {
            let elements: Vec<Box<Expression>>= raw_elements.iter()
                .map(|x| resolve_expression(x, variables, builtins))
                .collect();

            let supertype = get_common_supertype(
                &elements.iter().map(|x| x.result_type.as_ref().unwrap()).collect()
            ).clone();

            Expression {
                operation: Box::new(ExpressionOperation::ArrayLiteral(elements)),
                result_type: Some(supertype)
            }
        },
        abstract_syntax::Expression::NAryOperator(operator, arguments) => {
            let operator_function = match operator {
                abstract_syntax::NAryOperator::EqualTo => &builtins.operators.equal_to,
                abstract_syntax::NAryOperator::NotEqualTo => &builtins.operators.not_equal_to,
                abstract_syntax::NAryOperator::GreaterThan => &builtins.operators.greater_than,
                abstract_syntax::NAryOperator::GreaterThanOrEqualTo => &builtins.operators.greater_than_or_equal_to,
                abstract_syntax::NAryOperator::LesserThan => &builtins.operators.lesser_than,
                abstract_syntax::NAryOperator::LesserThanOrEqualTo => &builtins.operators.lesser_than_or_equal_to,
            };

            if arguments.len() < 2 {
                // We shouldn't get here if the parser works.
                panic!("N-Ary operator needs at least two arguments.");
            }
            else if arguments.len() > 2 {
                // TODO We can probably do this the easiest way by introducing a
                //  helper function for each n-ary operator. It would accept (*args) and
                //  simply do a for loop with window of size two, comparing elements.
                //  The alternative, in-function evaluation, would mean introducing new temp variables,
                //  which is certainly possible in a compiler but not easily in a transpiler.
                // Transitive properties of arguments should ensure validity.
                panic!("N-Ary operators support only two arguments (for now).");
            }

            resolve_static_function_call(operator_function, &arguments.iter().collect(), variables, builtins)
        },
        abstract_syntax::Expression::BinaryOperator(lhs, operator, rhs) => {
            let operator_function = match operator {
                abstract_syntax::BinaryOperator::Or => &builtins.operators.or,
                abstract_syntax::BinaryOperator::And => &builtins.operators.and,
                abstract_syntax::BinaryOperator::Multiply => &builtins.operators.multiply,
                abstract_syntax::BinaryOperator::Divide => &builtins.operators.divide,
                abstract_syntax::BinaryOperator::Add => &builtins.operators.add,
                abstract_syntax::BinaryOperator::Subtract => &builtins.operators.subtract,
                abstract_syntax::BinaryOperator::Exponentiate => &builtins.operators.exponentiate,
                abstract_syntax::BinaryOperator::Modulo => &builtins.operators.modulo,
            };

            resolve_static_function_call(operator_function, &vec![lhs, rhs], variables, builtins)
        },
        abstract_syntax::Expression::UnaryOperator(operator, expression) => {
            todo!()
        },
        abstract_syntax::Expression::VariableLookup(identifier) => {
            let variable = variables.resolve(identifier);

            Expression {
                operation: Box::new(ExpressionOperation::VariableLookup(variable.clone())),
                result_type: Some(variable.type_declaration.clone())
            }
        },
        abstract_syntax::Expression::FunctionCall(call_type, callee, arguments) => {
            if call_type == &abstract_syntax::FunctionCallType::Subscript {
                panic!("Subscript not supported yet");
            }

            let callee = resolve_expression(callee, variables, builtins);
            let arguments = arguments.iter()
                .enumerate()
                .map(|(idx, x)| Box::new(PassedArgument {
                    key: x.key.as_ref()
                        .map(|key| resolve_parameter_key(&key, idx))
                        .unwrap_or_else(|| ParameterKey::Int(idx as i32)),
                    value: resolve_expression(&x.value, variables, builtins)
                }))
                .collect();

            match callee.operation.as_ref() {
                ExpressionOperation::VariableLookup(variable) => {
                    if let Type::Function(function) = &variable.type_declaration.as_ref() {
                        // Can translate to static call!
                        return Box::new(Expression {
                            result_type: function.return_type.clone(),
                            operation: Box::new(ExpressionOperation::StaticFunctionCall {
                                function: Rc::clone(function),
                                arguments
                            })
                        })
                    }

                    panic!("Cannot call '{}'. It is not a function; dynamic calls are not yet supported.", &variable.name)
                }
                _ => panic!("Cannot call a non-function; dynamic calls are not yet supported.")
            }
        }
        abstract_syntax::Expression::MemberLookup(_, _) => {
            todo!()
        }
    })
}

pub fn resolve_type(syntax: &abstract_syntax::TypeDeclaration) -> Box<Type> {
    Box::new(match syntax.borrow() {
        abstract_syntax::TypeDeclaration::Identifier(id) => {
            use PrimitiveType::*;
            match id.as_str() {
                "Bool" => Type::Primitive(Bool),
                "Int8" => Type::Primitive(Int8),
                "Int16" => Type::Primitive(Int16),
                "Int32" => Type::Primitive(Int32),
                "Int64" => Type::Primitive(Int64),
                "Int128" => Type::Primitive(Int128),
                "UInt8" => Type::Primitive(UInt8),
                "UInt16" => Type::Primitive(UInt16),
                "UInt32" => Type::Primitive(UInt32),
                "UInt64" => Type::Primitive(UInt64),
                "UInt128" => Type::Primitive(UInt128),
                "Float32" => Type::Primitive(Float32),
                "Float64" => Type::Primitive(Float64),
                _ => Type::Identifier(id.clone())
            }
        },
        abstract_syntax::TypeDeclaration::NDArray(identifier, _) => {
            Type::NDArray(resolve_type(&identifier))
        }
    })
}
