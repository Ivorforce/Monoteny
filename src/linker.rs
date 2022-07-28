pub mod computation_tree;
pub mod builtins;

use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use uuid::Uuid;

use crate::abstract_syntax;
use crate::abstract_syntax::Opcode;
use crate::linker::computation_tree::*;
use crate::linker::builtins::*;

pub struct ScopedVariables<'a> {
    pub scopes: Vec<&'a HashMap<String, Rc<Variable>>>,
}

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

                functions_with_bodies.push((Rc::clone(&interface), &function.body));

                // Create a variable for the function
                global_variables.insert(interface.name.clone(), Rc::new(Variable {
                    id: Uuid::new_v4(),
                    name: interface.name.clone(),
                    type_declaration: Box::new(Type::Function(Rc::clone(&interface)))
                }));
            }
            abstract_syntax::GlobalStatement::Extension(_) => {
                todo!()
            }
        }
    }

    let global_variable_scope = builtin_variable_scope.subscope(&global_variables);

    // Resolve function bodies
    let mut functions: Vec<Rc<Function>> = functions_with_bodies.into_iter().map(
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

    for parameter in function.parameters.iter() {
        let variable = Rc::new(Variable {
            id: Uuid::new_v4(),
            name: parameter.internal_name.clone(),
            type_declaration: resolve_type(parameter.param_type.as_ref()),
        });

        parameters.push(Box::new(Parameter {
            external_key: ParameterKey::String(parameter.external_name.clone()),
            variable
        }));
    }

    return Rc::new(FunctionInterface {
        id: Uuid::new_v4(),
        name: function.identifier.clone(),
        parameters,
        return_type
    });
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
                });

                statements.push(Box::new(
                    Statement::VariableAssignment(Rc::clone(&variable), expression)
                ));
                local_variables.insert(variable.name.clone(), variable);
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
            _ => todo!()
        }
    }

    return Rc::new(Function {
        interface,
        variables: local_variables.values().map(|variable| (variable.id, variable.clone())).collect(),
        statements,
    });
}

pub fn resolve_expression(syntax: &abstract_syntax::Expression, variables: &ScopedVariables, builtins: &TenLangBuiltins) -> Box<Expression> {
    Box::new(match syntax {
        abstract_syntax::Expression::Number(n) => Expression {
            // TODO The type should be inferred
            operation: Box::new(ExpressionOperation::NumberLiteral(NumberLiteral::Int32(n.clone()))),
            result_type: Some(Box::new(Type::Identifier(String::from("Int32"))))
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

            let supertype = resolve_common_supertype(
                &elements.iter().map(|x| x.result_type.as_ref().unwrap()).collect()
            ).clone();

            Expression {
                operation: Box::new(ExpressionOperation::ArrayLiteral(elements)),
                result_type: Some(supertype)
            }
        },
        abstract_syntax::Expression::BinaryOperator(lhs_raw, operator, rhs_raw) => {
            let lhs = resolve_expression(lhs_raw, variables, builtins);
            let rhs = resolve_expression(rhs_raw, variables, builtins);

            // TODO This is obviously bullshit, but we don't have static operators with concrete return types yet.
            if &lhs.result_type.clone() != &rhs.result_type.clone() {
                panic!("binary operator sides must be of the same result type")
            }
            let result_type = lhs.result_type.clone();

            let operator_function = Rc::clone(match operator {
                Opcode::Multiply => &builtins.operators.multiply,
                Opcode::Divide => &builtins.operators.divide,
                Opcode::Add => &builtins.operators.add,
                Opcode::Subtract => &builtins.operators.subtract,
            });

            Expression {
                operation: Box::new(ExpressionOperation::StaticFunctionCall {
                    arguments: vec![
                        Box::new(PassedArgument { key: operator_function.parameters[0].external_key.clone(), value: lhs }),
                        Box::new(PassedArgument { key: operator_function.parameters[1].external_key.clone(), value: rhs }),
                    ],
                    function: operator_function,
                }),
                result_type
            }
        },
        abstract_syntax::Expression::VariableLookup(identifier) => {
            let variable = variables.resolve(identifier.clone());

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
                .map(|x| Box::new(PassedArgument {
                    key: x.name.clone().map(|x| ParameterKey::String(x)).unwrap_or_else(|| ParameterKey::Keyless),
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
                }
                _ => {}
            }

            // No static function call, must be dynamic!
            panic!("Dynamic function calls not yet implemented!")
        }
        abstract_syntax::Expression::MemberLookup(_, _) => {
            todo!()
        }
        abstract_syntax::Expression::Error => {
            todo!()
        }
    })
}

pub fn resolve_common_supertype<'a>(types: &Vec<&'a Box<Type>>) -> &'a Box<Type> {
    if types.is_empty() {
        panic!("Empty (inferred) array types are not supported for now.");
    }

    let reference = types[0];
    for other in types.iter().skip(1) {
        if *other != reference {
            panic!("Supertype inferral is not supported yet.")
        }
    }

    return reference;
}

pub fn resolve_type(syntax: &abstract_syntax::TypeDeclaration) -> Box<Type> {
    Box::new(match syntax.borrow() {
        abstract_syntax::TypeDeclaration::Identifier(id) => Type::Identifier(id.clone()),
        abstract_syntax::TypeDeclaration::NDArray(identifier, _) => {
            Type::NDArray(resolve_type(&identifier))
        }
    })
}

impl <'a> ScopedVariables<'a> {
    pub fn resolve(&self, variable_name: String) -> Rc<Variable> {
        for scope in self.scopes.iter() {
            if let Some(variable) = scope.get(&variable_name) {
                return variable.clone()
            }
        }

        panic!("Variable '{}' could not be resolved", variable_name)
    }

    pub fn subscope(&self, new_scope: &'a HashMap<String, Rc<Variable>>) -> ScopedVariables<'a> {
        let mut scopes: Vec<&'a HashMap<String, Rc<Variable>>> = Vec::new();

        scopes.push(new_scope);
        scopes.extend(self.scopes.iter());

        ScopedVariables { scopes }
    }
}
