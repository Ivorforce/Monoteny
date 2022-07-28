pub mod computation_tree;
pub mod builtins;

use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use uuid::Uuid;

use crate::abstract_syntax;
use crate::semantic_analysis::computation_tree::*;
use crate::semantic_analysis::builtins::*;

pub struct ScopedVariables<'a> {
    pub scopes: Vec<&'a HashMap<String, Rc<Variable>>>,
}

pub fn analyze_program(syntax: abstract_syntax::Program) -> Program {
    let mut functions: Vec<Box<Function>> = Vec::new();

    let (builtins, builtins_variables) = create_builtins();
    let global_variable_scope = ScopedVariables {
        scopes: vec![&builtins_variables],
    };

    for statement in syntax.global_statements {
        match *statement {
            abstract_syntax::GlobalStatement::FunctionDeclaration(function) => {
                functions.push(analyze_function(&function, &global_variable_scope));
            }
            abstract_syntax::GlobalStatement::Extension(extension) => {
                // TODO
            }
        }
    }

    return Program {
        variables: HashMap::new(),
        functions,
        builtins
    }
}

pub fn analyze_function(function: &abstract_syntax::Function, global_variables: &ScopedVariables) -> Box<Function> {
    let return_type = function.return_type.as_ref().map(|x| analyze_type(&x));

    let mut local_variables: HashMap<String, Rc<Variable>> = HashMap::new();

    let mut parameters: Vec<Box<Parameter>> = Vec::new();

    for parameter in function.parameters.iter() {
        let variable = Rc::new(Variable {
            id: Uuid::new_v4(),
            home: VariableHome::Local,
            name: parameter.internal_name.clone(),
            type_declaration: analyze_type(parameter.param_type.as_ref()),
        });

        local_variables.insert(variable.name.clone(), variable.clone());
        parameters.push(Box::new(Parameter {
            external_name: parameter.external_name.clone(),
            variable
        }));
    }

    // println!("{:?}", local_variables.borrow().keys());
    // println!("{:?}", variables.scopes[0].borrow().keys());

    let mut statements: Vec<Box<Statement>> = Vec::new();

    for statement in function.body.iter() {
        match statement.as_ref() {
            abstract_syntax::Statement::VariableDeclaration {
                mutability, identifier, type_declaration, expression
            } => {
                let variables = global_variables.subscope(&local_variables);
                let expression: Box<Expression> = analyze_expression(&expression, &variables);
                let inferred_type = expression.result_type;

                if let Some(type_declaration) = type_declaration {
                    let type_declaration = analyze_type(type_declaration);
                    if type_declaration != inferred_type {
                        panic!("Declared type of variable '{}' is not equal to inferred type '{:?}'", identifier, inferred_type);
                    }
                }

                let variable = Rc::new(Variable {
                    id: Uuid::new_v4(),
                    home: VariableHome::Local,
                    name: identifier.clone(),
                    type_declaration: inferred_type,
                });
                local_variables.insert(variable.name.clone(), variable);
            },
            abstract_syntax::Statement::Return(None) => {
                statements.push(Box::new(Statement::Return(None)));
            },
            abstract_syntax::Statement::Return(Some(expression)) => {
                let variables = global_variables.subscope(&local_variables);
                let expression: Box<Expression> = analyze_expression(&expression, &variables);

                if let Some(return_type) = return_type.clone() {
                    if return_type != expression.result_type {
                        panic!("Declared type of return statement is not equal to function return type '{:?}'", expression.result_type);
                    }
                }
                else {
                    panic!("return statement offers a value when the function does not return anything.");
                }

                statements.push(Box::new(Statement::Return(Some(expression))));
            }
            _ => {}
        }
    }

    return Box::new(Function {
        identifier: function.identifier.clone(),
        parameters,
        variables: local_variables.values().map(|variable| (variable.id, variable.clone())).collect(),
        statements,
        return_type
    });
}

pub fn analyze_expression(syntax: &abstract_syntax::Expression, variables: &ScopedVariables) -> Box<Expression> {
    Box::new(match syntax {
        abstract_syntax::Expression::Number(n) => Expression {
            operation: Box::new(ExpressionOperation::Number(n.clone())),
            // TODO Numbers should be postfixed with the type for now, maybe later inferred
            result_type: Box::new(Type::Identifier(String::from("Int32")))
        },
        abstract_syntax::Expression::StringLiteral(string) => {
            Expression {
                operation: Box::new(ExpressionOperation::StringLiteral(string.clone())),
                result_type: Box::new(Type::Identifier(String::from("String")))
            }
        },
        abstract_syntax::Expression::ArrayLiteral(raw_elements) => {
            let elements= raw_elements.iter()
                .map(|x| analyze_expression(x, variables))
                .collect();

            let supertype = common_supertype(&elements);

            Expression {
                operation: Box::new(ExpressionOperation::ArrayLiteral(elements)),
                result_type: supertype
            }
        },
        abstract_syntax::Expression::BinaryOperator(lhs_raw, operator, rhs_raw) => {
            let lhs = analyze_expression(lhs_raw, variables);
            let rhs = analyze_expression(rhs_raw, variables);

            // TODO This is obviously bullshit, but we don't have static operators yet.
            if &lhs.result_type.clone() != &rhs.result_type.clone() {
                panic!("binary operator sides must be of the same result type")
            }
            let result_type = lhs.result_type.clone();

            let function_expression = Expression {
                operation: Box::new(ExpressionOperation::VariableLookup(variables.resolve(format!("{:?}", operator)))),
                // TODO Functions should be typed individually
                result_type: Box::new(Type::Identifier(String::from("Function")))
            };

            Expression {
                operation: Box::new(ExpressionOperation::FunctionCall(
                    Box::new(function_expression),
                    vec![
                        Box::new(PassedArgument { name: None, value: lhs }),
                        Box::new(PassedArgument { name: None, value: rhs }),
                    ]
                )),
                result_type
            }
        },
        abstract_syntax::Expression::VariableLookup(identifier) => {
            let variable = variables.resolve(identifier.clone());

            Expression {
                operation: Box::new(ExpressionOperation::VariableLookup(variable.clone())),
                result_type: variable.type_declaration.clone()
            }
        },
        _ => todo!()
    })
}

pub fn common_supertype(expressions: &Vec<Box<Expression>>) -> Box<Type> {
    todo!()
}

pub fn analyze_type(syntax: &abstract_syntax::TypeDeclaration) -> Box<Type> {
    Box::new(match syntax.borrow() {
        abstract_syntax::TypeDeclaration::Identifier(id) => Type::Identifier(id.clone()),
        abstract_syntax::TypeDeclaration::NDArray(identifier, _) => {
            Type::NDArray(analyze_type(&identifier))
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
