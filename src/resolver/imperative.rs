use std::ops::Range;
use std::rc::Rc;

use itertools::{Either, Itertools};
use itertools::Either::{Left, Right};
use uuid::Uuid;

use crate::ast;
use crate::error::{ErrInRange, RResult, RuntimeError, TryCollectMany};
use crate::parser::expressions;
use crate::program::allocation::ObjectReference;
use crate::program::debug::MockFunctionInterface;
use crate::program::expression_tree::{ExpressionID, ExpressionOperation};
use crate::program::functions::{FunctionCallExplicity, FunctionHead, FunctionOverload, FunctionRepresentation, FunctionTargetType, ParameterKey};
use crate::program::generics::GenericAlias;
use crate::program::primitives;
use crate::program::traits::{Trait, TraitGraph};
use crate::program::types::*;
use crate::resolver::ambiguous::{AmbiguityResult, AmbiguousAbstractCall, AmbiguousFunctionCall, AmbiguousFunctionCandidate, ResolverAmbiguity};
use crate::resolver::imperative_builder::ImperativeBuilder;
use crate::resolver::scopes;
use crate::resolver::structs::AnonymousStruct;
use crate::resolver::type_factory::TypeFactory;
use crate::util::position::Positioned;

pub struct ImperativeResolver<'a> {
    pub builder: ImperativeBuilder<'a>,
    pub return_type: Rc<TypeProto>,
    pub ambiguities: Vec<Box<dyn ResolverAmbiguity>>,
}

impl <'a> ImperativeResolver<'a> {
    pub fn resolve_all_ambiguities(&mut self) -> RResult<()> {
        let mut has_changed = true;
        while !self.ambiguities.is_empty() {
            if !has_changed {
                return Err(
                    RuntimeError::error(format!("Function is ambiguous ({} times)", self.ambiguities.len()).as_str())
                        .with_notes(
                            self.ambiguities.iter()
                                .map(|x| RuntimeError::warning(x.to_string().as_str()).in_range(x.get_position()))
                        )
                        .to_array()
                );
            }

            has_changed = false;

            let callbacks: Vec<Box<dyn ResolverAmbiguity>> = self.ambiguities.drain(..).collect();
            for mut ambiguity in callbacks {
                match ambiguity.attempt_to_resolve(self)? {
                    AmbiguityResult::Ok(_) => has_changed = true,
                    AmbiguityResult::Ambiguous => self.ambiguities.push(ambiguity),
                }
            }
        }

        if !self.builder.type_factory.requirements.is_empty() {
            panic!()
        }

        Ok(())
    }

    pub fn register_ambiguity(&mut self, mut ambiguity: Box<dyn ResolverAmbiguity>) -> RResult<()> {
        match ambiguity.attempt_to_resolve(self)? {
            AmbiguityResult::Ok(_) => {},
            AmbiguityResult::Ambiguous => self.ambiguities.push(ambiguity),
        }

        Ok(())
    }

    pub fn resolve_abstract_function_call(&mut self, arguments: Vec<ExpressionID>, interface: Rc<Trait>, abstract_function: Rc<FunctionHead>, traits: TraitGraph, range: Range<usize>) -> RResult<ExpressionID> {
        let expression_id = self.builder.make_expression(arguments.clone());

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

    pub fn resolve_block(&mut self, body: &ast::Block, scope: &scopes::Scope) -> RResult<ExpressionID> {
        let mut scope = scope.subscope();
        let statements: Vec<ExpressionID> = body.statements.iter().map(|pstatement| {
            self.resolve_statement(&mut scope, pstatement)
                .err_in_range(&pstatement.value.position)
        }).try_collect()?;
        // try_collect means we stop after the first error.
        // This makes sense because an error may mean ambiguities or lacks of variable declarations.
        // Anything after the first error could just be a followup error.

        Ok(self.builder.make_operation_expression(statements, ExpressionOperation::Block))
    }

    fn resolve_statement(&mut self, scope: &mut scopes::Scope, pstatement: &ast::Decorated<Positioned<ast::Statement>>) -> RResult<ExpressionID> {
        let expression_id = match &pstatement.value.value {
            ast::Statement::VariableDeclaration {
                mutability, identifier, type_declaration, assignment
            } => {
                pstatement.no_decorations()?;

                let Some(assignment) = assignment else {
                    return Err(
                        RuntimeError::error(format!("Value {} must be assigned on declaration.", identifier).as_str()).to_array()
                    )
                };
                let assignment: ExpressionID = self.resolve_expression(&assignment, &scope)?;

                if let Some(type_declaration) = type_declaration {
                    self.hint_type(assignment, type_declaration, &scope)?;
                }

                let object_ref = Rc::new(ObjectReference { id: Uuid::new_v4(), type_: TypeProto::unit(TypeUnit::Generic(assignment)), mutability: mutability.clone() });
                self.builder.register_local(identifier, Rc::clone(&object_ref), scope)?;

                self.builder.make_full_expression(vec![assignment], &TypeProto::void(), ExpressionOperation::SetLocal(object_ref))?
            },
            ast::Statement::VariableUpdate { target, new_value } => {
                pstatement.no_decorations()?;

                let new_value: ExpressionID = self.resolve_expression(new_value, &scope)?;

                let lhs = expressions::parse(target, &scope.grammar)?;
                match &lhs.value {
                    expressions::Value::Identifier(identifier) => {
                        let object_ref = scope
                            .resolve(FunctionTargetType::Global, identifier)?
                            .as_local(true)?;
                        self.builder.types.bind(new_value, &object_ref.type_)?;

                        self.builder.make_full_expression(vec![new_value], &TypeProto::void(), ExpressionOperation::SetLocal(Rc::clone(&object_ref)))?
                    }
                    expressions::Value::MemberAccess(target, member) => {
                        let target = self.resolve_expression_token(target, scope)?;
                        let overload = scope
                            .resolve(FunctionTargetType::Member, &member)?
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
                    _ => return Err(RuntimeError::error("upd keyword must be followed by an identifier or a single member.").to_array())
                }
            }
            ast::Statement::Return(expression) => {
                pstatement.no_decorations()?;

                if let Some(expression) = expression {
                    if self.return_type.unit.is_void() {
                        return Err(
                            RuntimeError::error("Return statement offers a value when the function declares void.").to_array()
                        )
                    }

                    let result: ExpressionID = self.resolve_expression(expression, &scope)?;
                    self.builder.types.bind(result, &self.return_type)?;

                    self.builder.make_full_expression(vec![result], &TypeProto::void(), ExpressionOperation::Return)?
                } else {
                    if !self.return_type.unit.is_void() {
                        return Err(
                            RuntimeError::error("Return statement offers no value when the function declares an object.").to_array()
                        )
                    }

                    self.builder.make_full_expression(vec![], &TypeProto::void(), ExpressionOperation::Return)?
                }
            },
            ast::Statement::Expression(expression) => {
                pstatement.no_decorations()?;

                self.resolve_expression(&expression, &scope)?
            }
            statement => {
                return Err(
                    RuntimeError::error(format!("Statement {} is not supported in an imperative context.", statement).as_str()).to_array()
                )
            }
        };
        Ok(expression_id)
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
        let token = expressions::parse(syntax, &scope.grammar)?;
        self.resolve_expression_token(&token, scope)
            .err_in_range(&token.position)
    }

    pub fn resolve_expression_token(&mut self, ptoken: &Positioned<expressions::Value<Rc<FunctionHead>>>, scope: &scopes::Scope) -> RResult<ExpressionID> {
        let range = &ptoken.position;

        match &ptoken.value {
            expressions::Value::Operation(function_head, args) => {
                let args: Vec<_> = args.into_iter().map(|arg|
                    self.resolve_expression_token(&arg, scope)
                        .err_in_range(&arg.position)
                ).try_collect_many()?;

                self.resolve_function_call(
                    [function_head].into_iter(),
                    function_head.declared_representation.clone(),
                    vec![ParameterKey::Positional; args.len()],
                    args,
                    scope,
                    range.clone()
                )
            }
            expressions::Value::MacroIdentifier(identifier) => {
                return Err(RuntimeError::error("Macro not supported here.").to_array())
            }
            expressions::Value::Identifier(identifier) => {
                match self.resolve_global(scope, range, identifier)? {
                    Left(exp) => Ok(exp),
                    Right(fun) => self.builder.add_function_reference(&fun),
                }
            }
            expressions::Value::RealLiteral(s) => {
                let string_expression_id = self.builder.add_string_primitive(s)?;

                self.resolve_abstract_function_call(
                    vec![string_expression_id],
                    Rc::clone(&self.builder.runtime.traits.as_ref().unwrap().ConstructableByRealLiteral),
                    Rc::clone(&self.builder.runtime.traits.as_ref().unwrap().parse_real_literal_function),
                    scope.trait_conformance.clone(),
                    range.clone(),
                )
            }
            expressions::Value::IntLiteral(s) => {
                let string_expression_id = self.builder.add_string_primitive(s)?;

                self.resolve_abstract_function_call(
                    vec![string_expression_id],
                    Rc::clone(&self.builder.runtime.traits.as_ref().unwrap().ConstructableByIntLiteral),
                    Rc::clone(&self.builder.runtime.traits.as_ref().unwrap().parse_int_literal_function),
                    scope.trait_conformance.clone(),
                    range.clone(),
                )
            }
            expressions::Value::StringLiteral(parts) => {
                self.resolve_string_literal(scope, &range, parts)
            }
            expressions::Value::StructLiteral(struct_) => {
                let struct_ = self.resolve_struct(scope, struct_)?;

                if struct_.values.len() == 1 && struct_.keys[0] == ParameterKey::Positional {
                    return Ok(struct_.values[0])
                }

                return Err(RuntimeError::error("Anonymous struct literals are not yet supported.").to_array())
            }
            expressions::Value::ArrayLiteral(array) => {
                let values = array.arguments.iter().map(|x| {
                    self.resolve_expression_with_type(&x.value.value, &x.value.type_declaration, scope)
                        .err_in_range(&x.position)
                }).try_collect_many()?;

                let supertype = self.builder.types.merge_all(&values)?.clone();
                return Err(RuntimeError::error("Array literals are not yet supported.").to_array());
            }
            expressions::Value::Block(block) => {
                self.resolve_block(block, scope)
            }
            expressions::Value::MemberAccess(target, member) => {
                let target = self.resolve_expression_token(&target, scope)
                    .err_in_range(&target.position)?;

                match self.resolve_member(scope, range, member, target)? {
                    Left(expr) => Ok(expr),
                    Right(overload) => {
                        return Err(RuntimeError::error("Member function references are not yet supported.").to_array());
                    }
                }
            }
            expressions::Value::FunctionCall(call_target, struct_) => {
                let struct_ = self.resolve_struct(scope, struct_)?;

                // Check if we can do a direct function call
                let target_expression = match &call_target.value {
                    expressions::Value::Identifier(identifier) => {
                        // Found an identifier target. We may just be calling a global function!
                        match self.resolve_global(scope, range, identifier)? {
                            Left(expr) => expr, // It was more complicated after all.
                            Right(overload) => {
                                // It IS a function reference. Let's shortcut and call it directly.
                                return self.resolve_function_call(
                                    overload.functions.iter(),
                                    overload.representation.clone(),
                                    struct_.keys,
                                    struct_.values,
                                    scope,
                                    range.clone(),
                                )
                            }
                        }
                    }
                    expressions::Value::MemberAccess(member_target, member) => {
                        // Found a member access. We may just be calling a member function!

                        let target_expression = self.resolve_expression_token(&member_target, scope)
                            .err_in_range(&member_target.position)?;

                        match self.resolve_member(scope, &call_target.position, member, target_expression)? {
                            Left(expr) => expr, // It was more complicated after all.
                            Right(overload) => {
                                // It IS a member function reference. Let's shortcut and call it directly.
                                return self.resolve_function_call(
                                    overload.functions.iter(),
                                    overload.representation.clone(),
                                    [&ParameterKey::Positional].into_iter().chain(&struct_.keys).cloned().collect(),
                                    [&target_expression].into_iter().chain(&struct_.values).cloned().collect(),
                                    scope,
                                    range.clone(),
                                )
                            }
                        }
                    }
                    _ => {
                        self.resolve_expression_token(&call_target, scope)
                            .err_in_range(&call_target.position)?
                    }
                };

                // The call target is something more complicated. We'll call it as a function.

                let overload = scope
                    .resolve(FunctionTargetType::Member, "call_as_function")?
                    .as_function_overload()?;

                self.resolve_function_call(
                    overload.functions.iter(),
                    overload.representation.clone(),
                    [&ParameterKey::Positional].into_iter().chain(&struct_.keys).cloned().collect(),
                    [&target_expression].into_iter().chain(&struct_.values).cloned().collect(),
                    scope,
                    range.clone(),
                )
            }
            expressions::Value::Subscript(target, array) => {
                let values: Vec<_> = array.arguments.iter().map(|x| {
                    self.resolve_expression_with_type(&x.value.value, &x.value.type_declaration, scope)
                        .err_in_range(&x.position)
                }).try_collect_many()?;

                return Err(RuntimeError::error("Object subscript is not yet supported.").to_array())
            }
            expressions::Value::IfThenElse(if_then_else) => {
                let condition: ExpressionID = self.resolve_expression(&if_then_else.condition, &scope)?;
                self.builder.types.bind(condition, &TypeProto::unit(TypeUnit::Struct(Rc::clone(&self.builder.runtime.primitives.as_ref().unwrap()[&primitives::Type::Bool]))))?;
                let consequent: ExpressionID = self.resolve_expression(&if_then_else.consequent, &scope)?;

                let mut arguments = vec![condition, consequent];

                if let Some(alternative) = &if_then_else.alternative {
                    let alternative: ExpressionID = self.resolve_expression(alternative, &scope)?;
                    self.builder.types.bind(alternative, &TypeProto::unit(TypeUnit::Generic(consequent)))?;
                    arguments.push(alternative);
                }

                self.builder.make_full_expression(arguments, &TypeProto::unit(TypeUnit::Generic(consequent)), ExpressionOperation::IfThenElse)
            }
        }
    }

    fn resolve_member(&mut self, scope: &scopes::Scope, range: &Range<usize>, member: &str, target: ExpressionID) -> RResult<Either<ExpressionID, Rc<FunctionOverload>>> {
        let overload = scope.resolve(FunctionTargetType::Member, member)
            .err_in_range(range)?
            .as_function_overload().err_in_range(range)?;

        Ok(match overload.representation.call_explicity {
            FunctionCallExplicity::Explicit => {
                Right(overload)
            }
            FunctionCallExplicity::Implicit => {
                Left(self.resolve_function_call(
                    overload.functions.iter(),
                    overload.representation.clone(),
                    vec![ParameterKey::Positional],
                    vec![target],
                    scope,
                    range.clone()
                )?)
            }
        })
    }

    fn resolve_global(&mut self, scope: &scopes::Scope, range: &Range<usize>, identifier: &str) -> RResult<Either<ExpressionID, Rc<FunctionOverload>>> {
        Ok(match scope.resolve(FunctionTargetType::Global, identifier)? {
            scopes::Reference::Local(local) => {
                let ObjectReference { id, type_, mutability } = local.as_ref();

                Left(self.builder.make_full_expression(
                    vec![],
                    type_,
                    ExpressionOperation::GetLocal(local.clone())
                )?)
            }
            scopes::Reference::FunctionOverload(overload) => {
                match overload.representation.call_explicity {
                    FunctionCallExplicity::Explicit => {
                        Right(Rc::clone(overload))
                    }
                    FunctionCallExplicity::Implicit => {
                        Left(self.resolve_function_call(
                            overload.functions.iter(),
                            overload.representation.clone(),
                            vec![],
                            vec![],
                            scope,
                            range.clone()
                        )?)
                    }
                }
            }
        })
    }

    pub fn resolve_string_part(&mut self, part: &Positioned<ast::StringPart>, scope: &scopes::Scope) -> RResult<ExpressionID> {
        match &part.value {
            ast::StringPart::Literal(literal) => {
                self.builder.add_string_primitive(literal)
            },
            ast::StringPart::Object(o) => {
                let struct_ = self.resolve_struct(scope, o)?;
                // Call format(<args>)
                self.resolve_simple_function_call("format", struct_.keys, struct_.values, scope, part.position.clone())
            }
        }
    }

    pub fn resolve_string_literal(&mut self, scope: &scopes::Scope, range: &Range<usize>, parts: &Vec<Box<Positioned<ast::StringPart>>>) -> Result<ExpressionID, Vec<RuntimeError>> {
        Ok(match &parts[..] {
            // Simple case: Just one part means we can use it directly.
            [] => self.resolve_string_part(
                &Positioned {
                    position: range.clone(),
                    value: ast::StringPart::Literal("".to_string()),
                },
                scope
            )?,
            [part] => self.resolve_string_part(part, scope)?,
            _ => {
                let mut parts: Vec<_> = parts.iter()
                    .map(|part| self.resolve_string_part(part, scope))
                    .try_collect_many()?;

                // TODO We should call concat() with an array instead.
                let last = parts.pop().unwrap();
                parts.into_iter().try_rfold(last, |rstring, lstring| {
                    // Call format(<args>)
                    self.resolve_simple_function_call(
                        "add",
                        vec![ParameterKey::Positional, ParameterKey::Positional],
                        vec![lstring, rstring],
                        scope,
                        range.clone()
                    )
                })?
            }
        })
    }

    pub fn resolve_struct(&mut self, scope: &scopes::Scope, struct_: &ast::Struct) -> RResult<AnonymousStruct> {
        let values = struct_.arguments.iter().map(|x| {
            self.resolve_expression_with_type(&x.value.value, &x.value.type_declaration, scope)
                .err_in_range(&x.position)
        }).try_collect_many()?;

        Ok(AnonymousStruct {
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
                        self.builder.types.bind(expression_id, &TypeProto::unit_struct(&self.builder.runtime.traits.as_ref().unwrap().String))?;
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

            let generic_map = fun.interface.generics.iter()
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
            let expression_id = self.builder.make_expression(argument_expressions.clone());

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
            types: &self.builder.types,
        };

        let mut error = RuntimeError::error(
            format!("function {} could not be resolved.", signature).as_str());

        match &candidates_with_failed_signature[..] {
            [candidate] => {
                error = error.with_note(
                    RuntimeError::info(format!("Candidate has mismatching signature: {:?}", candidate).as_str())
                );
            }
            [] => {}
            candidates => {
                // TODO Output all?
                error = error.with_note(
                    RuntimeError::info(format!("{} candidates have mismatching signatures.", candidates.len()).as_str())
                );
            }
        }

        return Err(error.to_array());
    }

    pub fn hint_type(&mut self, value: GenericAlias, type_declaration: &ast::Expression, scope: &scopes::Scope) -> RResult<()> {
        let type_declaration = self.builder.type_factory.resolve_type(&type_declaration,true, &mut self.builder.runtime)?;

        for (name, generic) in self.builder.type_factory.generics.iter() {
            // need a mut scope
            // also, we'd need to insert it into our types
            panic!("Anonymous type hints are not supported yet")
            // scope.insert_singleton(
            //     scopes::Environment::Global,
            //     Reference::make_immutable_type(TypeProto::unit(generic.clone())),
            //     &name
            // );
        }

        self.builder.types.bind(value, type_declaration.as_ref())?;
        Ok(())
    }
}
