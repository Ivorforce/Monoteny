use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use uuid::Uuid;
use guard::guard;
use itertools::{Itertools, zip_eq};
use strum::IntoEnumIterator;
use crate::linker;
use crate::program::computation_tree::{ExpressionForest, ExpressionID, ExpressionOperation, Statement};
use crate::linker::global::link_type;
use crate::linker::{LinkError, scopes};
use crate::linker::ambiguous::{AmbiguousExpression, ExpressionCandidate};
use crate::parser::abstract_syntax;
use crate::program::allocation::{Mutability, Variable};
use crate::program::builtins::TenLangBuiltins;
use crate::program::functions::{FunctionPointer, FunctionPointerTarget, HumanFunctionInterface, MachineFunctionInterface, ParameterKey};
use crate::program::generics::{GenericAlias, TypeError, TypeForest};
use crate::program::global::FunctionImplementation;
use crate::program::primitives;
use crate::program::traits::{TraitBinding, TraitConformanceDeclaration, TraitConformanceRequirement};
use crate::program::types::*;

pub struct ImperativeLinker<'a> {
    pub function: Rc<FunctionPointer>,

    pub builtins: &'a TenLangBuiltins,

    pub expressions: Box<ExpressionForest>,
    pub unfinished_expressions: Vec<AmbiguousExpression>,

    pub conformance_delegations: &'a HashMap<Rc<TraitConformanceRequirement>, Rc<TraitConformanceDeclaration>>,
    pub variable_names: HashMap<Rc<Variable>, String>,
}

impl <'a> ImperativeLinker<'a> {
    pub fn link_function_body(mut self, body: &Vec<Box<abstract_syntax::Statement>>, scope: &scopes::Hierarchy) -> Result<Rc<FunctionImplementation>, LinkError> {
        let mut parameter_variables = scopes::Level::new();

        for (internal_name, (_, variable)) in zip_eq(self.function.human_interface.parameter_names_internal.iter(), self.function.human_interface.parameter_names.iter()) {
            parameter_variables.insert_singleton(scopes::Environment::Global, variable.clone(), internal_name);
        }

        let subscope = scope.subscope(&parameter_variables);
        let statements: Vec<Box<Statement>> = self.link_top_scope(body, &subscope)?;

        let mut has_changed = true;
        while !self.unfinished_expressions.is_empty() && has_changed {
            has_changed = false;

            let callbacks: Vec<AmbiguousExpression> = self.unfinished_expressions.drain(..).collect();
            for mut callback in callbacks {
                if callback.reduce(&mut self.expressions) {
                    has_changed = true;
                }
                else {
                    self.unfinished_expressions.push(callback);
                }
            }
        }
        if !self.unfinished_expressions.is_empty() {
            panic!("Failed resolving all generics.")
        }

        Ok(Rc::new(FunctionImplementation {
            implementation_id: self.function.pointer_id,
            function_id: match self.function.target {
                FunctionPointerTarget::Static { implementation_id } => implementation_id,
                _ => panic!()
            },
            human_interface: Rc::clone(&self.function.human_interface),
            machine_interface: Rc::clone(&self.function.machine_interface),
            statements,
            expression_forest: self.expressions,
            variable_names: self.variable_names.clone(),
            conformance_delegations: self.conformance_delegations.clone()
        }))
    }

    pub fn link_top_scope(&mut self, body: &Vec<Box<abstract_syntax::Statement>>, scope: &scopes::Hierarchy) -> Result<Vec<Box<Statement>>, LinkError> {
        if self.function.machine_interface.return_type.unit.is_void() {
            if let [statement] = &body[..] {
                if let abstract_syntax::Statement::Expression(expression ) = statement.as_ref() {
                    // Single-Statement Return
                    return Ok(vec![
                        Box::new(Statement::Return(Some(self.link_expression(expression, &scope)?)))
                    ])
                }
            }
        }

        self.link_scope(body, &scope)
    }

    pub fn link_unambiguous_expression(&mut self, arguments: Vec<ExpressionID>, return_type: &TypeProto, operation: ExpressionOperation) -> Result<ExpressionID, LinkError> {
        let id = self.expressions.register_new_expression(arguments);

        self.expressions.operations.insert(id.clone(), operation);

        LinkError::map(self.expressions.type_forest.bind(id, &return_type))
            .map(|_| id)
    }

    pub fn link_ambiguous_expression<I>(&mut self, arguments: Vec<ExpressionID>, candidates: I) -> Result<ExpressionID, LinkError> where I: Iterator<Item=ExpressionCandidate> {
        let id = self.expressions.register_new_expression(arguments);

        let mut ambiguous = AmbiguousExpression {
            expression_id: id,
            candidates: candidates.collect()
        };

        match ambiguous.reduce(&mut self.expressions) {
            true => {} // We're done, wasn't all that ambiguous after all!
            false => self.unfinished_expressions.push(ambiguous)  // Need to resolve the rest later.
        }

        Ok(id)
    }

    pub fn link_primitive<I>(&mut self, values: I) -> Result<ExpressionID, LinkError> where I: Iterator<Item=primitives::Value> {
        self.link_ambiguous_expression(
            vec![],
            values.into_iter()
                .map(|v| {
                    let type_proto = TypeProto::unit(TypeUnit::Primitive(v.get_type()));
                    let f: ExpressionCandidate =
                        Box::new(move |forest: &mut TypeForest, id: GenericAlias| -> Result<ExpressionOperation, LinkError> {
                            return LinkError::map(forest.bind(id, type_proto.as_ref()))
                                .map(|_| ExpressionOperation::Primitive(v))
                        });
                    f
                })
        )
    }

    pub fn link_scope(&mut self, body: &Vec<Box<abstract_syntax::Statement>>, scope: &scopes::Hierarchy) -> Result<Vec<Box<Statement>>, LinkError> {
        let mut local_variables = scopes::Level::new();
        let mut statements: Vec<Box<Statement>> = Vec::new();

        for statement in body.iter() {
            match statement.as_ref() {
                abstract_syntax::Statement::VariableDeclaration {
                    mutability, identifier, type_declaration, expression
                } => {
                    let subscope = scope.subscope(&local_variables);
                    let new_value: ExpressionID = self.link_expression(&expression, &subscope)?;

                    if let Some(type_declaration) = type_declaration {
                        let type_declaration = link_type(&type_declaration, &subscope);
                        LinkError::map(self.expressions.type_forest.bind(new_value, type_declaration.as_ref()))?;
                    }

                    let variable = Rc::new(Variable {
                        id: Uuid::new_v4(),
                        type_declaration: TypeProto::unit(TypeUnit::Generic(new_value)),
                        mutability: mutability.clone(),
                    });

                    statements.push(Box::new(
                        Statement::VariableAssignment(Rc::clone(&variable), new_value)
                    ));
                    self.variable_names.insert(Rc::clone(&variable), identifier.clone());
                    local_variables.push_variable(scopes::Environment::Global, variable, identifier);
                },
                abstract_syntax::Statement::Return(expression) => {
                    let subscope = scope.subscope(&local_variables);

                    if let Some(expression) = expression {
                        if self.function.machine_interface.return_type.unit.is_void() {
                            panic!("Return statement offers a value when the function declares void.")
                        }

                        let result: ExpressionID = self.link_expression(expression.as_ref(), &subscope)?;

                        LinkError::map(self.expressions.type_forest.bind(result, &self.function.machine_interface.return_type.as_ref()))?;
                        statements.push(Box::new(Statement::Return(Some(result))));
                    }
                    else {
                        if !self.function.machine_interface.return_type.unit.is_void() {
                            panic!("Return statement offers no value when the function declares an object.")
                        }

                        statements.push(Box::new(Statement::Return(None)));
                    }
                },
                abstract_syntax::Statement::Expression(expression) => {
                    let subscope = scope.subscope(&local_variables);
                    let expression: ExpressionID = self.link_expression(&expression, &subscope)?;
                    statements.push(Box::new(Statement::Expression(expression)));
                }
                abstract_syntax::Statement::VariableAssignment { variable_name, new_value } => {
                    let subscope = scope.subscope(&local_variables);
                    let variable = subscope.resolve_unambiguous(scopes::Environment::Global, variable_name);

                    if variable.mutability == Mutability::Immutable {
                        panic!("Cannot assign to immutable variable '{}'.", variable_name);
                    }

                    let new_value: ExpressionID = self.link_expression(&new_value, &subscope)?;
                    LinkError::map(self.expressions.type_forest.bind(new_value, variable.type_declaration.as_ref()))?;

                    statements.push(Box::new(
                        Statement::VariableAssignment(Rc::clone(&variable), new_value)
                    ));
                }
            }
        }

        Ok(statements)
    }

    pub fn link_expression(&mut self, syntax: &abstract_syntax::Expression, scope: &scopes::Hierarchy) -> Result<ExpressionID, LinkError> {
        match syntax {
            abstract_syntax::Expression::Int(string) => {
                self.link_primitive(
                    primitives::Type::iter()
                        .filter(primitives::Type::is_number)
                        .flat_map(|t| Some(t.parse_value(string)?))
                )
            },
            abstract_syntax::Expression::Float(string) => {
                self.link_primitive(
                    primitives::Type::iter()
                        .filter(primitives::Type::is_float)
                        .flat_map(|t| Some(t.parse_value(string)?))
                )
            },
            abstract_syntax::Expression::Bool(n) => {
                let value = primitives::Value::Bool(n.clone());

                self.link_unambiguous_expression(
                    vec![],
                    &TypeProto::unit(TypeUnit::Primitive(primitives::Type::Bool)),
                    ExpressionOperation::Primitive(value)
                )
            },
            abstract_syntax::Expression::StringLiteral(string) => {
                self.link_unambiguous_expression(
                    vec![],
                    &TypeProto::unit(TypeUnit::Struct(Rc::clone(&self.builtins.structs.String))),
                    ExpressionOperation::StringLiteral(string.clone())
                )
            },
            abstract_syntax::Expression::ArrayLiteral(raw_elements) => {
                let elements: Vec<ExpressionID>= raw_elements.iter()
                    .map(|x| self.link_expression(x, scope))
                    .try_collect()?;

                // TODO Wrap error if it occurs
                let supertype = self.expressions.type_forest.merge_all(&elements).unwrap().clone();

                self.link_unambiguous_expression(
                    vec![],
                    &TypeProto::monad(TypeProto::unit(TypeUnit::Generic(supertype))),
                    ExpressionOperation::ArrayLiteral
                )
            },
            abstract_syntax::Expression::BinaryOperator { lhs, operator, rhs } => {
                let lhs = self.link_expression(lhs, scope)?;
                let rhs = self.link_expression(rhs, scope)?;
                self.link_binary_function(lhs, operator, rhs, scope)
            },
            abstract_syntax::Expression::ConjunctivePairOperators { arguments, operators } => {
                todo!()
                // let arguments: Vec<Box<Expression>> = arguments.into_iter()
                //     .map(|x| self.link_expression(x, scope))
                //     .collect();
                //
                // let functions: Vec<Rc<FunctionInterface>> = zip_eq(arguments.windows(2), operators.into_iter())
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
                let argument = self.link_expression(argument, scope)?;
                self.link_unary_function(operator, argument, scope)
            },
            abstract_syntax::Expression::VariableLookup(identifier) => {
                let variable = scope.resolve_unambiguous(scopes::Environment::Global, identifier);

                self.link_unambiguous_expression(
                    vec![],
                    &variable.type_declaration,
                    ExpressionOperation::VariableLookup(variable.clone())
                )
            },
            abstract_syntax::Expression::FunctionCall { call_type, callee, arguments } => {
                if call_type == &abstract_syntax::FunctionCallType::Subscript {
                    panic!("Subscript not supported yet");
                }

                return match callee.as_ref() {
                    abstract_syntax::Expression::VariableLookup(function_name) => {
                        // Static Call
                        let argument_keys: Vec<ParameterKey> = arguments.iter()
                            .map(|arg| arg.key.clone())
                            .collect();

                        let argument_expressions: Vec<ExpressionID> = arguments.iter()
                            .map(|arg| self.link_expression(&arg.value, scope))
                            .try_collect()?;

                        self.link_function_call(function_name, scopes::Environment::Global, argument_keys, argument_expressions, scope)
                    }
                    _ => {
                        match callee.as_ref() {
                            abstract_syntax::Expression::MemberLookup { target, member_name } => {
                                let target = self.link_expression(target, scope);

                                // Member Function
                                let argument_keys: Vec<ParameterKey> = Some(ParameterKey::Positional).into_iter()
                                    .chain(arguments.iter().map(|arg| arg.key.clone()))
                                    .collect();

                                let argument_expressions: Vec<ExpressionID> = Some(target).into_iter()
                                    .chain(arguments.iter().map(|arg| self.link_expression(&arg.value, scope)))
                                    .try_collect()?;

                                self.link_function_call(member_name, scopes::Environment::Member, argument_keys, argument_expressions, scope)
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

    pub fn link_binary_function<'b>(&mut self, lhs: ExpressionID, operator: &'b String, rhs: ExpressionID, scope: &'b scopes::Hierarchy) -> Result<ExpressionID, LinkError> {
        self.link_function_call(operator, scopes::Environment::Global, vec![ParameterKey::Positional, ParameterKey::Positional], vec![lhs, rhs], scope)
    }

    pub fn link_unary_function<'b>(&mut self, operator: &'b String, value: ExpressionID, scope: &'b scopes::Hierarchy) -> Result<ExpressionID, LinkError> {
        self.link_function_call(operator, scopes::Environment::Global, vec![ParameterKey::Positional], vec![value], scope)
    }

    pub fn link_function_call(&mut self, fn_name: &String, environment: scopes::Environment, argument_keys: Vec<ParameterKey>, argument_expressions: Vec<ExpressionID>, scope: &scopes::Hierarchy) -> Result<ExpressionID, LinkError> {
        // TODO Check if any arguments are void before anything else
        let seed = Uuid::new_v4();

        let functions = scope.resolve_functions(environment, fn_name);

        let argument_keys: Vec<&ParameterKey> = argument_keys.iter().collect();

        let mut candidates_with_failed_signature = vec![];
        let mut candidates: Vec<ExpressionCandidate> = vec![];

        // Need to cache this because the candidates need to access it statically later.
        let argument_expressions = Rc::new(argument_expressions);
        let conformance = Rc::new(scope.trait_conformance_declarations.clone());

        for fun in functions.into_iter().map(Rc::clone) {
            let param_keys = fun.human_interface.parameter_names.iter().map(|x| &x.0).collect::<Vec<&ParameterKey>>();
            if param_keys != argument_keys {
                candidates_with_failed_signature.push(fun);
                continue;
            }

            let param_types: Vec<Box<TypeProto>> = fun.human_interface.parameter_names.iter()
                .map(|x| x.1.type_declaration.with_any_as_generic(&seed))
                .collect();

            // Movable reference copy.
            let argument_expressions = Rc::clone(&argument_expressions);
            let conformance = Rc::clone(&conformance);
            let argument_targets: Rc<Vec<Rc<Variable>>> = Rc::new(
                fun.human_interface.parameter_names.iter()
                    .map(|x| Rc::clone(&x.1))
                    .collect()
            );
            let fun = Rc::clone(&fun);

            candidates.push(Box::new(move |types: &mut TypeForest, id: ExpressionID| {
                for (arg, param) in zip_eq(
                    argument_expressions.iter(),
                    param_types.iter().map(|x| x.as_ref())
                ) {
                    LinkError::map(types.bind(arg.clone(), param))?;
                }
                let e = types.bind(id.clone(), &fun.machine_interface.return_type);
                LinkError::map(e)?;

                let binding = LinkError::map_trait_error(
                    conformance
                        .satisfy_requirements(&fun.machine_interface.requirements, &seed, &types)
                )?;

                Ok(ExpressionOperation::FunctionCall {
                    function: Rc::clone(&fun),
                    argument_targets: argument_targets.as_ref().clone(),
                    binding
                })
            }));
        }

        if candidates.len() >= 1 {
            return self.link_ambiguous_expression(argument_expressions.as_ref().clone(), candidates.into_iter());
        }

        // TODO We should probably output the locations of candidates.

        if candidates_with_failed_signature.len() > 1 {
            panic!("function {} could not be resolved. {} candidates have mismatching arguments: {:?}", fn_name, candidates_with_failed_signature.len(), argument_keys)
        }

        if candidates_with_failed_signature.len() == 1 {
            // TODO Print passed arguments like a signature, not array
            let candidate = candidates_with_failed_signature.iter().next().unwrap();
            panic!("function {:?} could not be resolved. Candidate has mismatching arguments: {:?}", candidate.human_interface, argument_keys)
        }

        panic!("function {} could not be resolved.", fn_name)
    }
}
