use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;

use itertools::Itertools;
use uuid::Uuid;

use crate::error::{ErrInRange, RResult, RuntimeError};
use crate::interpreter::runtime::Runtime;
use crate::resolver::ambiguous::{AmbiguityResult, AmbiguousAbstractCall, AmbiguousFunctionCall, AmbiguousFunctionCandidate, ResolverAmbiguity};
use crate::resolver::grammar::parse::{resolve_expression_to_tokens, resolve_tokens_to_value};
use crate::resolver::grammar::Struct;
use crate::resolver::scopes;
use crate::resolver::type_factory::TypeFactory;
use crate::parser::ast;
use crate::program::allocation::ObjectReference;
use crate::program::calls::FunctionBinding;
use crate::program::debug::MockFunctionInterface;
use crate::program::expression_tree::{ExpressionID, ExpressionOperation, ExpressionTree};
use crate::program::{function_object, primitives};
use crate::program::function_object::{FunctionCallExplicity, FunctionOverload, FunctionRepresentation, FunctionTargetType};
use crate::program::functions::{FunctionHead, ParameterKey};
use crate::program::generics::{GenericAlias, TypeForest};
use crate::program::global::FunctionImplementation;
use crate::program::traits::{RequirementsAssumption, Trait, TraitConformanceRule, TraitGraph};
use crate::program::types::*;
use crate::util::position::Positioned;

pub struct ImperativeResolver<'a> {
    pub function: Rc<FunctionHead>,

    pub runtime: &'a Runtime,

    pub types: Box<TypeForest>,
    pub expression_tree: Box<ExpressionTree>,
    pub ambiguities: Vec<Box<dyn ResolverAmbiguity>>,

    pub locals_names: HashMap<Rc<ObjectReference>, String>,
}

impl <'a> ImperativeResolver<'a> {
    pub fn register_new_expression(&mut self, arguments: Vec<ExpressionID>) -> ExpressionID {
        let id = ExpressionID::new_v4();

        self.types.register(id);
        for argument in arguments.iter() {
            self.expression_tree.parents.insert(*argument, id);
        }
        self.expression_tree.children.insert(id, arguments);

        id
    }

    pub fn resolve_function_body(mut self, body: &ast::Expression, scope: &scopes::Scope) -> RResult<Box<FunctionImplementation>> {
        let mut scope = scope.subscope();

        let granted_requirements = scope.trait_conformance.assume_granted(
            self.function.interface.requirements.iter().cloned()
        );

        // Let our scope know that our parameter types (all of type any!) conform to the requirements
        for conformance in granted_requirements.iter() {
            scope.trait_conformance.add_conformance_rule(TraitConformanceRule::direct(
                Rc::clone(conformance),
            ));
        };

        // Add abstract function mocks to our scope to be callable.
        for conformance in granted_requirements.iter() {
            for (abstract_function, function) in conformance.function_mapping.iter() {
                // TODO Do we need to keep track of the object reference created by this trait conformance?
                //  For the record, it SHOULD be created - an abstract function reference can still be passed around,
                //  assigned and maybe called later.
                scope.overload_function(
                    function,
                    conformance.binding.trait_.abstract_functions[abstract_function].clone(),
                )?;
            }
        }

        // TODO Register generic types as variables so they can be referenced in the function

        // Register parameters as variables.
        let mut parameter_variables = vec![];
        for parameter in self.function.interface.parameters.clone() {
            let parameter_variable = ObjectReference::new_immutable(parameter.type_.clone());
            _ = self.register_local(&parameter.internal_name, Rc::clone(&parameter_variable), &mut scope);
            parameter_variables.push(parameter_variable);
        }

        let head_expression = self.resolve_expression(body, &scope)?;
        self.types.bind(head_expression, &self.function.interface.return_type)?;
        self.expression_tree.root = head_expression;  // TODO This is kinda dumb; but we can't write into an existing head expression

        let mut has_changed = true;
        while !self.ambiguities.is_empty() {
            if !has_changed {
                // TODO Output which parts are ambiguous, and how, by asking the objects
                return Err(RuntimeError::new(
                    format!("Ambiguous ({} times): \n{}\n\n", self.ambiguities.len(), self.ambiguities.iter().map(|x| x.to_string()).join("\n"))
                ))
            }

            has_changed = false;

            let callbacks: Vec<Box<dyn ResolverAmbiguity>> = self.ambiguities.drain(..).collect();
            for mut ambiguity in callbacks {
                match ambiguity.attempt_to_resolve(&mut self)? {
                    AmbiguityResult::Ok(_) => has_changed = true,
                    AmbiguityResult::Ambiguous => self.ambiguities.push(ambiguity),
                }
            }
        }

        Ok(Box::new(FunctionImplementation {
            head: self.function,
            requirements_assumption: Box::new(RequirementsAssumption { conformance: HashMap::from_iter(granted_requirements.into_iter().map(|c| (Rc::clone(&c.binding), c))) }),
            expression_tree: self.expression_tree,
            type_forest: self.types,
            parameter_locals: parameter_variables,
            locals_names: self.locals_names,
        }))
    }

    pub fn resolve_unambiguous_expression(&mut self, arguments: Vec<ExpressionID>, return_type: &TypeProto, operation: ExpressionOperation) -> RResult<ExpressionID> {
        let id = self.register_new_expression(arguments);

        self.expression_tree.values.insert(id.clone(), operation);

        self.types.bind(id, &return_type)
            .map(|_| id)
    }

    pub fn register_ambiguity(&mut self, mut ambiguity: Box<dyn ResolverAmbiguity>) -> RResult<()> {
        match ambiguity.attempt_to_resolve(self)? {
            AmbiguityResult::Ok(_) => {},
            AmbiguityResult::Ambiguous => self.ambiguities.push(ambiguity),
        }

        Ok(())
    }

    pub fn resolve_abstract_function_call(&mut self, arguments: Vec<ExpressionID>, interface: Rc<Trait>, abstract_function: Rc<FunctionHead>, traits: TraitGraph, range: Range<usize>) -> RResult<ExpressionID> {
        let expression_id = self.register_new_expression(arguments.clone());

        self.register_ambiguity(Box::new(AmbiguousAbstractCall {
            expression_id,
            arguments,
            trait_: interface,
            range,
            abstract_function,
            traits,
        }))?;

        return Ok(expression_id);
    }

    pub fn hint_type(&mut self, value: GenericAlias, type_declaration: &ast::Expression, scope: &scopes::Scope) -> RResult<()> {
        let mut type_factory = TypeFactory::new(&scope, &self.runtime);

        let type_declaration = type_factory.resolve_type(&type_declaration, true)?;

        for requirement in type_factory.requirements {
            todo!("Implicit imperative requirements are not implemented yet")
        }

        for (name, generic) in type_factory.generics.into_iter() {
            panic!("Anonymous type hints are not supported yet") // need a mut scope
            // scope.insert_singleton(
            //     scopes::Environment::Global,
            //     Reference::make_immutable_type(TypeProto::unit(generic.clone())),
            //     &name
            // );
        }

        self.types.bind(value, type_declaration.as_ref())?;
        Ok(())
    }

    pub fn resolve_block(&mut self, body: &ast::Block, scope: &scopes::Scope) -> RResult<ExpressionID> {
        let mut scope = scope.subscope();
        let statements: Vec<ExpressionID> = body.statements.iter().map(|pstatement| {
            self.resolve_statement(&mut scope, pstatement)
                .err_in_range(&pstatement.value.position)
        }).try_collect()?;

        let expression_id = self.register_new_expression(statements);
        self.expression_tree.values.insert(expression_id, ExpressionOperation::Block);

        Ok(expression_id)
    }

    fn resolve_statement(&mut self, scope: &mut scopes::Scope, pstatement: &ast::Decorated<Positioned<ast::Statement>>) -> RResult<ExpressionID> {
        let expression_id = match &pstatement.value.value {
            ast::Statement::VariableDeclaration {
                mutability, identifier, type_declaration, assignment
            } => {
                pstatement.no_decorations()?;

                let Some(assignment) = assignment else {
                    return Err(RuntimeError::new(format!("Value {} must be assigned on declaration.", identifier)))
                };
                let assignment: ExpressionID = self.resolve_expression(&assignment, &scope)?;

                if let Some(type_declaration) = type_declaration {
                    self.hint_type(assignment, type_declaration, &scope)?;
                }

                let object_ref = Rc::new(ObjectReference { id: Uuid::new_v4(), type_: TypeProto::unit(TypeUnit::Generic(assignment)), mutability: mutability.clone() });
                self.register_local(identifier, Rc::clone(&object_ref), scope);

                let expression_id = self.register_new_expression(vec![assignment]);
                self.expression_tree.values.insert(expression_id, ExpressionOperation::SetLocal(object_ref));
                self.types.bind(expression_id, &TypeProto::void())?;
                expression_id
            },
            ast::Statement::VariableUpdate { target, new_value } => {
                pstatement.no_decorations()?;

                let new_value: ExpressionID = self.resolve_expression(&new_value, &scope)?;

                match &target.iter().map(|a| a.as_ref()).collect_vec()[..] {
                    [Positioned { position, value: ast::Term::Identifier(identifier) }] => {
                        let object_ref = scope
                            .resolve(function_object::FunctionTargetType::Global, identifier)?
                            .as_local(true)?;
                        self.types.bind(new_value, &object_ref.type_)?;
                        let expression_id = self.register_new_expression(vec![new_value]);
                        self.expression_tree.values.insert(expression_id, ExpressionOperation::SetLocal(Rc::clone(&object_ref)));
                        self.types.bind(expression_id, &TypeProto::void())?;
                        expression_id
                    }
                    [
                        ..,
                        Positioned { value: ast::Term::Dot, .. },
                        Positioned { value: ast::Term::Identifier(access), .. }
                    ] => {
                        let target = self.resolve_expression(&target[..target.len() - 2], scope)?;
                        let overload = scope
                            .resolve(function_object::FunctionTargetType::Member, &access)?
                            .as_function_overload()?;
                        self.resolve_function_call(
                            overload.functions.iter(),
                            overload.representation.clone(),
                            vec![ParameterKey::Positional, ParameterKey::Positional],
                            vec![target, new_value],
                            scope,
                            pstatement.value.position.clone()
                        )?
                    }
                    _ => return Err(RuntimeError::new("upd keyword must be followed by an identifier or a single member.".to_string()))
                }
            }
            ast::Statement::Return(expression) => {
                pstatement.no_decorations()?;

                if let Some(expression) = expression {
                    if self.function.interface.return_type.unit.is_void() {
                        return Err(RuntimeError::new(format!("Return statement offers a value when the function declares void.")))
                    }

                    let result: ExpressionID = self.resolve_expression(expression, &scope)?;
                    self.types.bind(result, &self.function.interface.return_type)?;

                    let expression_id = self.register_new_expression(vec![result]);
                    self.expression_tree.values.insert(expression_id, ExpressionOperation::Return);
                    self.types.bind(expression_id, &TypeProto::void())?;
                    expression_id
                } else {
                    if !self.function.interface.return_type.unit.is_void() {
                        return Err(RuntimeError::new(format!("Return statement offers no value when the function declares an object.")))
                    }

                    let expression_id = self.register_new_expression(vec![]);
                    self.expression_tree.values.insert(expression_id, ExpressionOperation::Return);
                    self.types.bind(expression_id, &TypeProto::void())?;
                    expression_id
                }
            },
            ast::Statement::Expression(expression) => {
                pstatement.no_decorations()?;

                self.resolve_expression(&expression, &scope)?
            }
            ast::Statement::IfThenElse(if_then_else) => {
                pstatement.no_decorations()?;

                let condition: ExpressionID = self.resolve_expression(&if_then_else.condition, &scope)?;
                self.types.bind(condition, &TypeProto::unit(TypeUnit::Struct(Rc::clone(&self.runtime.primitives.as_ref().unwrap()[&primitives::Type::Bool]))))?;
                let consequent: ExpressionID = self.resolve_expression(&if_then_else.consequent, &scope)?;

                let mut arguments = vec![condition, consequent];

                if let Some(alternative) = &if_then_else.alternative {
                    let alternative: ExpressionID = self.resolve_expression(alternative, &scope)?;
                    self.types.bind(alternative, &TypeProto::unit(TypeUnit::Generic(consequent)))?;
                    arguments.push(alternative);
                }

                let expression_id = self.register_new_expression(arguments);
                self.expression_tree.values.insert(expression_id, ExpressionOperation::IfThenElse);
                self.types.bind(expression_id, &TypeProto::unit(TypeUnit::Generic(consequent)))?;
                expression_id
            }
            statement => {
                return Err(RuntimeError::new(format!("Statement {} is not supported in an imperative context.", statement)))
            }
        };
        Ok(expression_id)
    }

    fn register_local(&mut self, identifier: &str, reference: Rc<ObjectReference>, scope: &mut scopes::Scope) {
        self.locals_names.insert(Rc::clone(&reference), identifier.to_string());
        scope.override_reference(function_object::FunctionTargetType::Global, scopes::Reference::Local(reference), identifier);
    }

    pub fn resolve_expression_with_type(&mut self, syntax: &ast::Expression, type_declaration: &Option<ast::Expression>, scope: &scopes::Scope) -> RResult<ExpressionID> {
        let value = self.resolve_expression(syntax, scope)?;
        if let Some(type_declaration) = type_declaration {
            self.hint_type(value, type_declaration, scope)?
        }
        Ok(value)
    }

    pub fn resolve_expression(&mut self, syntax: &[Box<Positioned<ast::Term>>], scope: &scopes::Scope) -> RResult<ExpressionID> {
        // First, resolve core grammar.
        let tokens = resolve_expression_to_tokens(self, syntax, scope)?;
        // Then, resolve configurable user grammar.
        resolve_tokens_to_value(tokens, scope, self)
    }

    pub fn resolve_string_primitive(&mut self, value: &str) -> RResult<ExpressionID> {
        self.resolve_unambiguous_expression(
            vec![],
            &TypeProto::unit_struct(&self.runtime.traits.as_ref().unwrap().String),
            ExpressionOperation::StringLiteral(value.to_string())
        )
    }

    pub fn resolve_string_part(&mut self, part: &Positioned<ast::StringPart>, scope: &scopes::Scope) -> RResult<ExpressionID> {
        match &part.value {
            ast::StringPart::Literal(literal) => {
                self.resolve_string_primitive(literal)
            },
            ast::StringPart::Object(o) => {
                let struct_ = self.resolve_struct(scope, o)?;
                // Call format(<args>)
                self.resolve_simple_function_call("format", struct_.keys, struct_.values, scope, part.position.clone())
            }
        }
    }

    pub fn resolve_string_literal(&mut self, scope: &scopes::Scope, ast_token: &Box<Positioned<ast::Term>>, parts: &Vec<Box<Positioned<ast::StringPart>>>) -> Result<ExpressionID, Vec<RuntimeError>> {
        Ok(match &parts[..] {
            // Simple case: Just one part means we can use it directly.
            [] => self.resolve_string_part(
                &ast_token.with_value(ast::StringPart::Literal("".to_string())),
                scope
            )?,
            [part] => self.resolve_string_part(part, scope)?,
            _ => {
                let mut parts: Vec<_> = parts.iter()
                    .map(|part| self.resolve_string_part(part, scope))
                    .try_collect()?;

                // TODO We should call concat() with an array instead.
                let last = parts.pop().unwrap();
                parts.into_iter().try_rfold(last, |rstring, lstring| {
                    // Call format(<args>)
                    self.resolve_simple_function_call(
                        "add",
                        vec![ParameterKey::Positional, ParameterKey::Positional],
                        vec![lstring, rstring],
                        scope,
                        ast_token.position.clone()
                    )
                })?
            }
        })
    }

    pub fn resolve_struct(&mut self, scope: &scopes::Scope, struct_: &ast::Struct) -> RResult<Struct> {
        let values = struct_.arguments.iter().map(|x| {
            self.resolve_expression_with_type(&x.value.value, &x.value.type_declaration, scope)
        }).try_collect()?;

        Ok(Struct {
            keys: struct_.arguments.iter()
                .map(|x| x.value.key.clone())
                .collect(),
            values,
        })
    }

    pub fn resolve_simple_function_call(&mut self, name: &str, keys: Vec<ParameterKey>, args: Vec<ExpressionID>, scope: &scopes::Scope, range: Range<usize>) -> RResult<ExpressionID> {
        let variable = scope.resolve(FunctionTargetType::Global, name)?;

        match variable {
            scopes::Reference::FunctionOverload(overload) => {
                match (overload.representation.target_type, overload.representation.call_explicity) {
                    (FunctionTargetType::Global, FunctionCallExplicity::Explicit) => {
                        let expression_id = self.resolve_function_call(overload.functions.iter(), overload.representation.clone(), keys, args, scope, range)?;
                        // Make sure the return type is actually String.
                        self.types.bind(expression_id, &TypeProto::unit_struct(&self.runtime.traits.as_ref().unwrap().String))?;
                        Ok(expression_id)
                    }
                    // this could happen if somebody uses def format ... without parentheses.
                    _ => panic!("'{}' must not be shadowed in this context.", name)
                }
            }
            // todo lolz, this is kinda dumb?
            _ => panic!("'{}' must not be shadowed in this context.", name)
        }
    }

    pub fn resolve_conjunctive_pairs(&mut self, arguments: Vec<Positioned<ExpressionID>>, operations: Vec<Rc<FunctionHead>>) -> RResult<Positioned<ExpressionID>> {
        todo!()
    }

    pub fn resolve_function_call<'b>(&mut self, functions: impl Iterator<Item=&'b Rc<FunctionHead>>, representation: FunctionRepresentation, argument_keys: Vec<ParameterKey>, argument_expressions: Vec<ExpressionID>, scope: &scopes::Scope, range: Range<usize>) -> RResult<ExpressionID> {
        // TODO Check if any arguments are void before anything else
        let argument_keys: Vec<&ParameterKey> = argument_keys.iter().collect();

        let mut candidates_with_failed_signature = vec![];
        let mut candidates: Vec<Box<AmbiguousFunctionCandidate>> = vec![];

        for fun in functions.map(Rc::clone) {
            let param_keys = fun.interface.parameters.iter().map(|x| &x.external_key).collect::<Vec<&ParameterKey>>();
            if param_keys != argument_keys {
                candidates_with_failed_signature.push(fun);
                continue;
            }

            let generic_map = fun.interface.generics.values()
                .map(|trait_| (Rc::clone(trait_), TypeProto::unit(TypeUnit::Generic(Uuid::new_v4()))))
                .collect();

            candidates.push(Box::new(AmbiguousFunctionCandidate {
                param_types: fun.interface.parameters.iter()
                    .map(|x| x.type_.replacing_structs(&generic_map))
                    .collect(),
                return_type: fun.interface.return_type.replacing_structs(&generic_map),
                requirements: fun.interface.requirements.iter().cloned().collect_vec(),
                function: fun,
                generic_map,
            }));
        }

        if candidates.len() >= 1 {
            let expression_id = self.register_new_expression(argument_expressions.clone());

            self.register_ambiguity(Box::new(AmbiguousFunctionCall {
                expression_id,
                representation,
                arguments: argument_expressions,
                traits: scope.trait_conformance.clone(),
                range,
                candidates,
                failed_candidates: vec![]
            }))?;

            return Ok(expression_id);
        }

        // TODO We should probably output the locations of candidates.

        let signature = MockFunctionInterface {
            representation,
            argument_keys: argument_keys.clone().into_iter().cloned().collect_vec(),
            arguments: argument_expressions.clone(),
            types: &self.types,
        };
        match &candidates_with_failed_signature[..] {
            [candidate] => {
                // TODO Print passed arguments like a signature, not array
                Err(RuntimeError::new(format!("function {} could not be resolved.\nCandidate has mismatching signature: {:?}", signature, candidate)))
            }
            [] => {
                Err(RuntimeError::new(format!("function {} could not be resolved.", signature)))
            }
            candidates => {
                Err(RuntimeError::new(format!("function {} could not be resolved.\n{} candidates have mismatching signatures.", signature, candidates.len())))
            }
        }
    }

    pub fn resolve_function_reference(&mut self, overload: &Rc<FunctionOverload>) -> RResult<ExpressionID> {
        match overload.functions.iter().exactly_one() {
            Ok(function) => {
                let getter = &self.runtime.source.fn_getters[function];
                let expression_id = self.resolve_unambiguous_expression(
                    vec![],
                    &getter.interface.return_type,
                    // Call the getter of the function 'object' instead of the function itself.
                    ExpressionOperation::FunctionCall(FunctionBinding::pure(Rc::clone(getter)))
                )?;

                Ok(expression_id)
            }
            _ => return Err(RuntimeError::new(
                String::from("References to overloaded functions are not yet supported (need syntax to distinguish which to choose).")
            ))?,
        }
    }
}
