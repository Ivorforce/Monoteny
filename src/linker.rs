pub mod builtins;
pub mod computation_tree;
pub mod scopes;
pub mod primitives;

use std::collections::HashMap;
use std::rc::Rc;
use guard::guard;
use uuid::Uuid;

use crate::abstract_syntax;
use crate::abstract_syntax::BinaryOperator;
use crate::linker::builtins::*;
use crate::linker::computation_tree::*;
use crate::linker::scopes::Scope;


pub fn resolve_program(syntax: abstract_syntax::Program) -> Program {
    let builtins = create_builtins();
    let builtin_variable_scope = Scope {
        scopes: vec![&builtins.global_constants]
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
        }
    }

    let global_variable_scope = builtin_variable_scope.subscope(&global_variables);

    // Resolve function bodies
    let functions: Vec<Rc<Function>> = functions_with_bodies.into_iter().map(
        |(interface, statements)|
        resolve_function_body(statements, &interface, &global_variable_scope)
    ).collect();

    return Program {
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

pub fn resolve_function_body(body: &Vec<Box<abstract_syntax::Statement>>, interface: &Rc<FunctionInterface>, scope: &Scope) -> Rc<Function> {
    let mut parameter_variables: HashMap<String, Rc<Variable>> = HashMap::new();

    for parameter in &interface.parameters {
        let variable = &parameter.variable;
        parameter_variables.insert(variable.name.clone(), variable.clone());
    }

    let subscope = scope.subscope(&parameter_variables);
    let statements: Vec<Box<Statement>> = resolve_top_scope(body, interface, &subscope);

    return Rc::new(Function {
        interface: Rc::clone(interface),
        statements,
    });
}

pub fn resolve_top_scope(body: &Vec<Box<abstract_syntax::Statement>>, interface: &Rc<FunctionInterface>, scope: &Scope) -> Vec<Box<Statement>> {
    if let Some(_) = &interface.return_type {
        if let [statement] = &body[..] {
            if let abstract_syntax::Statement::Expression(expression ) = statement.as_ref() {
                // Single-Statement Return
                return vec![Box::new(Statement::Return(Some(resolve_expression(expression, &scope))))]
            }
        }
    }

    resolve_scope(body, &interface, &scope)
}

pub fn resolve_scope(body: &Vec<Box<abstract_syntax::Statement>>, interface: &Rc<FunctionInterface>, scope: &Scope) -> Vec<Box<Statement>> {
    let mut local_variables: HashMap<String, Rc<Variable>> = HashMap::new();
    let mut statements: Vec<Box<Statement>> = Vec::new();

    for statement in body.iter() {
        match statement.as_ref() {
            abstract_syntax::Statement::VariableDeclaration {
                mutability, identifier, type_declaration, expression
            } => {
                let subscope = scope.subscope(&local_variables);
                let expression: Box<Expression> = resolve_expression(&expression, &subscope);
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
            abstract_syntax::Statement::Return(expression) => {
                let subscope = scope.subscope(&local_variables);
                let expression: Option<Box<Expression>> = expression.as_ref().map(|x| resolve_expression(x, &subscope));

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
                let expression: Box<Expression> = resolve_expression(&expression, &subscope);
                statements.push(Box::new(Statement::Expression(expression)));
            }
            abstract_syntax::Statement::VariableAssignment(name, expression) => {
                let subscope = scope.subscope(&local_variables);
                let variable = subscope.resolve(name);

                if variable.mutability == abstract_syntax::Mutability::Immutable {
                    panic!("Cannot assign to immutable variable '{}'.", name);
                }

                let expression: Box<Expression> = resolve_expression(&expression, &subscope);

                statements.push(Box::new(
                    Statement::VariableAssignment(Rc::clone(&variable), expression)
                ));
            }
        }
    }

    statements
}

pub struct BinaryPairNode<'a> {
    pub value: &'a abstract_syntax::Expression,
    pub next: Option<BinaryNeighborOperationNode<'a>>,
}

pub struct BinaryNeighborOperationNode<'a> {
    pub operator: &'a BinaryOperator,
    pub operand: Box<BinaryPairNode<'a>>
}

pub fn gather_comparison_pairs_left_associative<'a>(lhs: &'a abstract_syntax::Expression, operator: &'a BinaryOperator, rhs: &'a abstract_syntax::Expression) -> Box<BinaryPairNode<'a>> {
    if operator.is_pairwise_comparison() {
        match lhs {
            abstract_syntax::Expression::BinaryOperator(two_lhs, left_operator, lhs) => {
                return Box::new(BinaryPairNode {
                    value: rhs,
                    next: Some(BinaryNeighborOperationNode {
                        operator, operand: gather_comparison_pairs_left_associative(two_lhs, left_operator, lhs)
                    }),
                });
            }
            _ => { }
        };
    }

    return Box::new(BinaryPairNode {
        value: rhs,
        next: Some(BinaryNeighborOperationNode {
            operator,
            operand: Box::new(BinaryPairNode { value: lhs, next: None })
        }),
    });
}

pub fn resolve_static_function_call(function: &Rc<FunctionInterface>, arguments: Vec<Box<Expression>>, scope: &Scope) -> Box<Expression> {
    guard!(let Some(reference) = arguments.first() else {
        // TODO
        panic!("operators without arguments are not supported yet - because generic return types are not supported yet.")
    });

    for other in arguments.iter().skip(1) {
        if &other.result_type.clone() != &reference.result_type.clone() {
            // TODO
            panic!("operator sides must be of the same result type - because generic return types are not supported yet.")
        }
    }

    let result_type = reference.result_type.clone();

    Box::new(Expression {
        operation: Box::new(ExpressionOperation::StaticFunctionCall {
            arguments: arguments.into_iter()
                .enumerate()
                .map(|(idx, argument)| Box::new(PassedArgument { key: function.parameters[idx].external_key.clone(), value: argument }))
                .collect(),
            function: function.clone(),
        }),
        result_type
    })
}

pub fn resolve_expression(syntax: &abstract_syntax::Expression, scope: &Scope) -> Box<Expression> {
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
                result_type: Some(Box::new(Type::Identifier(String::from("String"))))
            })
        },
        abstract_syntax::Expression::ArrayLiteral(raw_elements) => {
            let elements: Vec<Box<Expression>>= raw_elements.iter()
                .map(|x| resolve_expression(x, scope))
                .collect();

            let supertype = get_common_supertype(
                &elements.iter().map(|x| x.result_type.as_ref().unwrap()).collect()
            ).clone();

            Box::new(Expression {
                operation: Box::new(ExpressionOperation::ArrayLiteral(elements)),
                result_type: Some(supertype)
            })
        },
        abstract_syntax::Expression::BinaryOperator(lhs, operator, rhs) => {
            let pairs = gather_comparison_pairs_left_associative(lhs, operator, rhs);

            if let None = pairs.next.unwrap().operand.next {
                // Just one pair, this is easy
                let function = scope.resolve_static_fn(&format!("{:?}", operator));
                let lhs = resolve_expression(lhs, scope);
                let rhs = resolve_expression(rhs, scope);

                resolve_static_function_call(function, vec![lhs, rhs], scope)
            }
            else {
                // At least 3 parts
                todo!()
            }
        },
        abstract_syntax::Expression::UnaryOperator(operator, expression) => {
            todo!()
        },
        abstract_syntax::Expression::VariableLookup(identifier) => {
            let variable = scope.resolve(identifier);

            Box::new(Expression {
                operation: Box::new(ExpressionOperation::VariableLookup(variable.clone())),
                result_type: Some(variable.type_declaration.clone())
            })
        },
        abstract_syntax::Expression::FunctionCall(call_type, callee, arguments) => {
            if call_type == &abstract_syntax::FunctionCallType::Subscript {
                panic!("Subscript not supported yet");
            }

            let callee = resolve_expression(callee, scope);
            let arguments = arguments.iter()
                .enumerate()
                .map(|(idx, x)| Box::new(PassedArgument {
                    key: x.key.as_ref()
                        .map(|key| resolve_parameter_key(&key, idx))
                        .unwrap_or_else(|| ParameterKey::Int(idx as i32)),
                    value: resolve_expression(&x.value, scope)
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
    }
}

pub fn resolve_type(syntax: &abstract_syntax::TypeDeclaration) -> Box<Type> {
    Box::new(match syntax {
        abstract_syntax::TypeDeclaration::Identifier(id) => {
            if let Some(primitive) = primitives::parse(id) {
                Type::Primitive(primitive)
            }
            else {
                Type::Identifier(id.clone())
            }
        },
        abstract_syntax::TypeDeclaration::NDArray(identifier, _) => {
            Type::NDArray(resolve_type(&identifier))
        }
    })
}
