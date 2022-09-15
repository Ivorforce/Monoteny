use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use uuid::Uuid;
use std::iter::zip;
use guard::guard;
use crate::linker;
use crate::linker::computation_tree::{Expression, ExpressionOperation, FunctionImplementation, Statement};
use crate::linker::global::{link_type};
use crate::linker::scopes;
use crate::parser::abstract_syntax;
use crate::program::builtins::TenLangBuiltins;
use crate::program::functions::{HumanFunctionInterface, MachineFunctionInterface};
use crate::program::generics::GenericMapping;
use crate::program::primitives;
use crate::program::traits::TraitBinding;
use crate::program::types::*;

pub struct ImperativeLinker<'a> {
    pub interface: &'a Rc<HumanFunctionInterface>,
    pub builtins: &'a TenLangBuiltins,
    pub generics: GenericMapping,
    pub variable_names: HashMap<Rc<Variable>, String>
}

impl <'a> ImperativeLinker<'a> {
    pub fn link_function_body(&mut self, body: &Vec<Box<abstract_syntax::Statement>>, scope: &scopes::Hierarchy) -> Rc<FunctionImplementation> {
        let mut parameter_variables = scopes::Level::new();

        for (internal_name, (_, variable)) in zip(self.interface.parameter_names_internal.iter(), self.interface.parameter_names.iter()) {
            parameter_variables.insert_singleton(scopes::Environment::Global, variable.clone(), internal_name);
        }

        let subscope = scope.subscope(&parameter_variables);
        let statements: Vec<Box<Statement>> = self.link_top_scope(body, &subscope);

        return Rc::new(FunctionImplementation {
            interface: Rc::clone(self.interface),
            statements,
            variable_names: self.variable_names.clone()
        });
    }

    pub fn link_top_scope(&mut self, body: &Vec<Box<abstract_syntax::Statement>>, scope: &scopes::Hierarchy) -> Vec<Box<Statement>> {
        if let Some(_) = &self.interface.machine_interface.return_type {
            if let [statement] = &body[..] {
                if let abstract_syntax::Statement::Expression(expression ) = statement.as_ref() {
                    // Single-Statement Return
                    return vec![Box::new(Statement::Return(Some(self.link_expression(expression, &scope))))]
                }
            }
        }

        self.link_scope(body, &scope)
    }

    pub fn link_scope(&mut self, body: &Vec<Box<abstract_syntax::Statement>>, scope: &scopes::Hierarchy) -> Vec<Box<Statement>> {
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
                        let type_declaration = link_type(&type_declaration, &subscope);
                        if &type_declaration != inferred_type {
                            panic!("Declared type of variable '{}' is not equal to inferred type '{:?}'", identifier, inferred_type);
                        }
                    }

                    let variable = Rc::new(Variable {
                        id: Uuid::new_v4(),
                        type_declaration: inferred_type.clone(),
                        mutability: mutability.clone(),
                    });

                    statements.push(Box::new(
                        Statement::VariableAssignment(Rc::clone(&variable), expression)
                    ));
                    local_variables.push_variable(scopes::Environment::Global, variable, identifier);
                },
                abstract_syntax::Statement::Return(expression) => {
                    let subscope = scope.subscope(&local_variables);
                    let expression: Option<Box<Expression>> = expression.as_ref().map(|x| self.link_expression(x, &subscope));

                    match (&self.interface.machine_interface.return_type, expression) {
                        (Some(_), None) => panic!("Return statement offers no value when the function declares an object."),
                        (None, Some(_)) => panic!("Return statement offers a value when the function declares void."),
                        (None, None) => {
                            statements.push(Box::new(Statement::Return(None)));
                        },
                        (Some(interface_return_type), Some(expression)) => {
                            match &expression.result_type {
                                None => panic!("Return statement expression resolves to void. Please move the expression into a separate line."),
                                Some(result_type) => {
                                    // TODO Do anything with minimal type?
                                    let _ = self.generics.merge(interface_return_type, result_type);
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

    pub fn link_expression(&mut self, syntax: &abstract_syntax::Expression, scope: &scopes::Hierarchy) -> Box<Expression> {
        match syntax {
            abstract_syntax::Expression::Number(n) => {
                // TODO The type should be inferred
                let value = primitives::Value::Int32(n.clone());
                Box::new(Expression {
                    result_type: Some(Type::unit(TypeUnit::Primitive(value.get_type()))),
                    operation: Box::new(ExpressionOperation::Primitive(value)),
                })
            },
            abstract_syntax::Expression::Bool(n) => Box::new(Expression {
                operation: Box::new(ExpressionOperation::Primitive(primitives::Value::Bool(n.clone()))),
                result_type: Some(Type::unit(TypeUnit::Primitive(primitives::Type::Bool)))
            }),
            abstract_syntax::Expression::StringLiteral(string) => {
                Box::new(Expression {
                    operation: Box::new(ExpressionOperation::StringLiteral(string.clone())),
                    result_type: Some(Type::unit(TypeUnit::Struct(Rc::clone(&self.builtins.structs.String))))
                })
            },
            abstract_syntax::Expression::ArrayLiteral(raw_elements) => {
                let elements: Vec<Box<Expression>>= raw_elements.iter()
                    .map(|x| self.link_expression(x, scope))
                    .collect();

                let supertype = self.generics.merge_all(
                    &elements.iter().map(|x| x.result_type.as_deref().unwrap()).collect()
                ).unwrap().clone();

                Box::new(Expression {
                    operation: Box::new(ExpressionOperation::ArrayLiteral(elements)),
                    result_type: Some(supertype)
                })
            },
            abstract_syntax::Expression::BinaryOperator { lhs, operator, rhs } => {
                let lhs = self.link_expression(lhs, scope);
                let rhs = self.link_expression(rhs, scope);
                self.link_binary_function(lhs, operator, rhs, scope)
            },
            abstract_syntax::Expression::ConjunctivePairOperators { arguments, operators } => {
                todo!()
                // let arguments: Vec<Box<Expression>> = arguments.into_iter()
                //     .map(|x| self.link_expression(x, scope))
                //     .collect();
                //
                // let functions: Vec<Rc<FunctionInterface>> = zip(arguments.windows(2), operators.into_iter())
                //     .map(|(args, operator)| {
                //         let (lhs, rhs) = (&args[0], &args[1]);
                //         self.link_binary_function(lhs, operator, rhs, scope).clone()
                //     })
                //     .collect();
                //
                // if arguments.len() != functions.len() + 1 || arguments.len() < 2 {
                //     panic!("Internal Error for PairAssociativeBinaryOperators: (args.len(): {}, functions.len(): {})", arguments.len(), functions.len());
                // }
                // else {
                //     if functions.len() == 1 {
                //         println!("Warning: Attempting making a pair-associative operator from just 1 pair. This should not happen.");
                //     }
                //
                //     Box::new(Expression {
                //         // TODO This is not true; we have to see what (a > b) && (b > c) actually outputs
                //         result_type: Some(Type::unit(TypeUnit::Primitive(primitives::Type::Bool))),
                //         operation: Box::new(ExpressionOperation::PairwiseOperations { arguments, functions })
                //     })
                // }
            }
            abstract_syntax::Expression::UnaryOperator { operator, argument} => {
                let argument = self.link_expression(argument, scope);
                self.link_unary_function(operator, argument, scope)
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
                        let arguments: Vec<(ParameterKey, Box<Expression>)> = arguments.iter()
                            .map(|arg| (arg.key.clone(), self.link_expression(&arg.value, scope)))
                            .collect();

                        self.link_function_call(function_name, scopes::Environment::Global, arguments, scope)
                    }
                    _ => {
                        match callee.as_ref() {
                            abstract_syntax::Expression::MemberLookup { target, member_name } => {
                                let target = self.link_expression(target, scope);

                                // Member Function
                                let arguments: Vec<(ParameterKey, Box<Expression>)> = Some((ParameterKey::None, target)).into_iter()
                                    .chain(arguments.iter().map(|x| (x.key.clone(), self.link_expression(&x.value, scope))))
                                    .collect();

                                self.link_function_call(member_name, scopes::Environment::Member, arguments, scope)
                            },
                            _ => {
                                // Function call on some object
                                todo!()
                                // let target = self.link_expression(callee, scope);
                                //
                                // let arguments: Vec<Box<PassedArgument>> = self.link_passed_arguments(arguments.iter(), scope, 0);
                                //
                                // let function = match &target.result_type {
                                //     Some(result_type) => {
                                //         match &result_type.unit {
                                //             TypeUnit::Function(function) => function,
                                //             _ => panic!("Expression does not resolve to a function."),
                                //         }
                                //     }
                                //     _ => panic!("Expression does not return anything."),
                                // };
                                //
                                // link_static_function_call(function, arguments)
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

    pub fn link_binary_function<'b>(&mut self, lhs: Box<Expression>, operator: &'b String, rhs: Box<Expression>, scope: &'b scopes::Hierarchy) -> Box<Expression> {
        guard!(let Some(lhs_type) = &lhs.result_type else {
            panic!("Left side of binary operator {} evaluates to void.", operator)
        });
        guard!(let Some(rhs_type) = &rhs.result_type else {
            panic!("Right side of binary operator {} evaluates to void.", operator)
        });

        self.link_function_call(operator, scopes::Environment::Global, vec![
            (ParameterKey::None, lhs),
            (ParameterKey::None, rhs),
        ], scope)
    }

    pub fn link_unary_function<'b>(&mut self, operator: &'b String, value: Box<Expression>, scope: &'b scopes::Hierarchy) -> Box<Expression> {
        guard!(let Some(arg_type) = &value.result_type else {
            panic!("Argument of unary operator {} evaluates to void.", operator)
        });

        self.link_function_call(operator, scopes::Environment::Global, vec![
            (ParameterKey::None, value),
        ], scope)
    }

    pub fn link_function_call(&mut self, fn_name: &String, environment: scopes::Environment, arguments: Vec<(ParameterKey, Box<Expression>)>, scope: &scopes::Hierarchy) -> Box<Expression> {
        let seed = Uuid::new_v4();

        let functions = scope.resolve_functions(environment, fn_name);

        let (argument_keys, argument_expressions): (Vec<ParameterKey>, Vec<Box<Expression>>) = arguments.into_iter().unzip();
        let argument_keys: Vec<&ParameterKey> = argument_keys.iter().collect();
        let argument_types: Vec<&Type> = argument_expressions.iter().map(|x| x.result_type.as_ref().unwrap().as_ref()).collect();

        let candidates: Vec<(Rc<HumanFunctionInterface>, Vec<Box<Type>>)> = functions.iter()
            .flat_map(|fun| {
                if fun.parameter_names.iter().map(|x| &x.0).collect::<Vec<&ParameterKey>>() != argument_keys {
                    return None
                }

                let types: Vec<Box<Type>> = fun.parameter_names.iter().map(|x| x.1.type_declaration.generify(&seed)).collect();
                return Some((Rc::clone(fun), types))
            })
            .collect();

        let candidates: HashMap<Rc<HumanFunctionInterface>, (Vec<Box<Type>>, Box<TraitBinding>)> = candidates.into_iter().flat_map(|(fun, params)| {
            let mut clone: GenericMapping = self.generics.clone();
            clone.merge_pairs(zip(
                argument_types.iter().map(|x| *x),
                params.iter().map(|x| x.as_ref())
            )).ok()?;
            let binding = scope.trait_conformance_declarations.satisfy_requirements(&fun.machine_interface.requirements, &seed, &clone).ok()?;
            Some((Rc::clone(&fun), (params, binding)))
        }).collect();

        // TODO Only allow abstract candidates in relation to abstract requirements declared in our current scope

        if candidates.len() == 0 {
            panic!("function could not be resolved for the passed arguments: {:?}", &argument_types)
        }
        else if candidates.len() > 1 {
            panic!("function is ambiguous ({}x) for the passed arguments: {:?}", candidates.len(), &argument_types)
        }
        else {
            let (function, (param_types, binding)) = candidates.into_iter().next().unwrap();
            let return_type = function.machine_interface.return_type.as_ref()
                .map(|x| x.generify(&seed));

            // Actually bind the generics w.r.t. the selected function
            self.generics.merge_pairs(zip(
                argument_types,
                param_types.iter().map(|x| x.as_ref())
            )).unwrap();

            return Box::new(Expression {
                result_type: return_type,
                operation: Box::new(ExpressionOperation::FunctionCall {
                    function: Rc::clone(&function),
                    arguments: zip(argument_expressions.into_iter(), function.parameter_names.iter())
                        .map(|(exp, (_, variable))| (Rc::clone(variable), exp))
                        .collect(),
                    binding
                })
            })
        }
    }
}
