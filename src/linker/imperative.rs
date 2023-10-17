use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;
use uuid::Uuid;
use guard::guard;
use itertools::Itertools;
use try_map::FallibleMapExt;
use crate::error::{ErrInRange, RResult, RuntimeError};
use crate::interpreter::Runtime;
use crate::program::computation_tree::{ExpressionID, ExpressionOperation, ExpressionTree};
use crate::linker::{precedence, scopes};
use crate::linker::ambiguous::{AmbiguityResult, AmbiguousAbstractCall, AmbiguousFunctionCall, AmbiguousFunctionCandidate, LinkerAmbiguity};
use crate::linker::precedence::link_patterns;
use crate::linker::type_factory::TypeFactory;
use crate::parser::ast;
use crate::program::allocation::{ObjectReference, Reference};
use crate::program::debug::MockFunctionInterface;
use crate::program::function_object::{FunctionForm, FunctionRepresentation};
use crate::program::functions::{FunctionHead, ParameterKey};
use crate::program::generics::{GenericAlias, TypeForest};
use crate::program::global::FunctionImplementation;
use crate::program::r#struct::Struct;
use crate::program::traits::{RequirementsAssumption, Trait, TraitConformanceRule, TraitGraph};
use crate::program::types::*;
use crate::util::position::Positioned;

pub struct ImperativeLinker<'a> {
    pub function: Rc<FunctionHead>,

    pub runtime: &'a Runtime,

    pub types: Box<TypeForest>,
    pub expressions: Box<ExpressionTree>,
    pub ambiguities: Vec<Box<dyn LinkerAmbiguity>>,

    pub locals_names: HashMap<Rc<ObjectReference>, String>,
}

impl <'a> ImperativeLinker<'a> {
    pub fn register_new_expression(&mut self, arguments: Vec<ExpressionID>) -> ExpressionID {
        let id = ExpressionID::new_v4();

        self.types.register(id);
        self.expressions.arguments.insert(id, arguments);

        id
    }

    pub fn link_function_body(mut self, body: &ast::Expression, scope: &scopes::Scope) -> RResult<Box<FunctionImplementation>> {
        let mut scope = scope.subscope();

        let granted_requirements = scope.traits.assume_granted(
            self.function.interface.requirements.iter().cloned()
        );

        // Let our scope know that our parameter types (all of type any!) conform to the requirements
        for conformance in granted_requirements.iter() {
            scope.traits.add_conformance_rule(TraitConformanceRule::direct(
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

        let head_expression = self.link_expression(body, &scope)?;
        self.types.bind(head_expression, &self.function.interface.return_type)?;

        let mut has_changed = true;
        while !self.ambiguities.is_empty() {
            if !has_changed {
                // TODO Output which parts are ambiguous, and how, by asking the objects
                return Err(RuntimeError::new(
                    format!("Ambiguous ({} times): \n{}\n\n", self.ambiguities.len(), self.ambiguities.iter().map(|x| x.to_string()).join("\n"))
                ))
            }

            has_changed = false;

            let callbacks: Vec<Box<dyn LinkerAmbiguity>> = self.ambiguities.drain(..).collect();
            for mut ambiguity in callbacks {
                match ambiguity.attempt_to_resolve(&mut self)? {
                    AmbiguityResult::Ok(_) => has_changed = true,
                    AmbiguityResult::Ambiguous => self.ambiguities.push(ambiguity),
                }
            }
        }

        Ok(Box::new(FunctionImplementation {
            implementation_id: self.function.function_id,
            head: self.function,
            requirements_assumption: Box::new(RequirementsAssumption { conformance: HashMap::from_iter(granted_requirements.into_iter().map(|c| (Rc::clone(&c.binding), c))) }),
            root_expression_id: head_expression,
            expression_forest: self.expressions,
            type_forest: self.types,
            parameter_locals: parameter_variables,
            locals_names: self.locals_names,
        }))
    }

    pub fn link_unambiguous_expression(&mut self, arguments: Vec<ExpressionID>, return_type: &TypeProto, operation: ExpressionOperation) -> RResult<ExpressionID> {
        let id = self.register_new_expression(arguments);

        self.expressions.operations.insert(id.clone(), operation);

        self.types.bind(id, &return_type)
            .map(|_| id)
    }

    pub fn register_ambiguity(&mut self, mut ambiguity: Box<dyn LinkerAmbiguity>) -> RResult<()> {
        match ambiguity.attempt_to_resolve(self)? {
            AmbiguityResult::Ok(_) => {},
            AmbiguityResult::Ambiguous => self.ambiguities.push(ambiguity),
        }

        Ok(())
    }

    pub fn link_abstract_function_call(&mut self, arguments: Vec<ExpressionID>, interface: Rc<Trait>, abstract_function: Rc<FunctionHead>, traits: TraitGraph, range: Range<usize>) -> RResult<ExpressionID> {
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

    pub fn link_string_literal(&mut self, value: &str) -> RResult<ExpressionID> {
        self.link_unambiguous_expression(
            vec![],
            &TypeProto::unit_struct(&self.runtime.traits.as_ref().unwrap().String),
            ExpressionOperation::StringLiteral(value.to_string())
        )
    }

    pub fn hint_type(&mut self, value: GenericAlias, type_declaration: &ast::Expression, scope: &scopes::Scope) -> RResult<()> {
        let mut type_factory = TypeFactory::new(&scope, &self.runtime);

        let type_declaration = type_factory.link_type(&type_declaration, true)?;

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

    pub fn link_block(&mut self, body: &Vec<Box<Positioned<ast::Statement>>>, scope: &scopes::Scope) -> RResult<ExpressionID> {
        let mut scope = scope.subscope();
        let statements: Vec<ExpressionID> = body.iter().map(|pstatement| {
            self.link_statement(&mut scope, pstatement)
                .err_in_range(&pstatement.position)
        }).try_collect()?;

        let expression_id = self.register_new_expression(statements);
        self.expressions.operations.insert(expression_id, ExpressionOperation::Block);

        Ok(expression_id)
    }

    fn link_statement(&mut self, scope: &mut scopes::Scope, pstatement: &Positioned<ast::Statement>) -> RResult<ExpressionID> {
        let expression_id = match &pstatement.value {
            ast::Statement::VariableDeclaration {
                mutability, identifier, type_declaration, assignment
            } => {
                guard!(let Some(assignment) = assignment else {
                    return Err(RuntimeError::new(format!("Value {} must be assigned on declaration.", identifier)))
                });
                let assignment: ExpressionID = self.link_expression(&assignment, &scope)?;

                if let Some(type_declaration) = type_declaration {
                    self.hint_type(assignment, type_declaration, &scope)?;
                }

                let object_ref = Rc::new(ObjectReference { id: Uuid::new_v4(), type_: TypeProto::unit(TypeUnit::Generic(assignment)), mutability: mutability.clone() });
                self.register_local(identifier, Rc::clone(&object_ref), scope);

                let expression_id = self.register_new_expression(vec![assignment]);
                self.expressions.operations.insert(expression_id, ExpressionOperation::SetLocal(object_ref));
                self.types.bind(expression_id, &TypeProto::void())?;
                expression_id
            },
            ast::Statement::MemberAssignment { access, new_value } => {
                let new_value: ExpressionID = self.link_expression(&new_value, &scope)?;

                let target = self.link_term(&access.target, scope)?;
                let target = link_patterns(vec![target], scope, self)?;
                let overload = scope
                    .resolve(scopes::Environment::Member, &access.member)?
                    .as_function_overload()?;
                self.link_function_call(overload.functions.iter(), overload.representation.clone(), vec![ParameterKey::Positional, ParameterKey::Positional], vec![target, new_value], scope, pstatement.position.clone())?
            }
            ast::Statement::LocalAssignment { identifier, new_value } => {
                let new_value: ExpressionID = self.link_expression(&new_value, &scope)?;

                let object_ref = scope
                    .resolve(scopes::Environment::Global, identifier)?
                    .as_local(true)?;
                self.types.bind(new_value, &object_ref.type_)?;
                let expression_id = self.register_new_expression(vec![new_value]);
                self.expressions.operations.insert(expression_id, ExpressionOperation::SetLocal(Rc::clone(&object_ref)));
                self.types.bind(expression_id, &TypeProto::void())?;
                expression_id
            }
            ast::Statement::Return(expression) => {
                if let Some(expression) = expression {
                    if self.function.interface.return_type.unit.is_void() {
                        return Err(RuntimeError::new(format!("Return statement offers a value when the function declares void.")))
                    }

                    let result: ExpressionID = self.link_expression(expression, &scope)?;
                    self.types.bind(result, &self.function.interface.return_type)?;

                    let expression_id = self.register_new_expression(vec![result]);
                    self.expressions.operations.insert(expression_id, ExpressionOperation::Return);
                    self.types.bind(expression_id, &TypeProto::void())?;
                    expression_id
                } else {
                    if !self.function.interface.return_type.unit.is_void() {
                        return Err(RuntimeError::new(format!("Return statement offers no value when the function declares an object.")))
                    }

                    let expression_id = self.register_new_expression(vec![]);
                    self.expressions.operations.insert(expression_id, ExpressionOperation::Return);
                    self.types.bind(expression_id, &TypeProto::void())?;
                    expression_id
                }
            },
            ast::Statement::Expression(expression) => {
                self.link_expression(&expression, &scope)?
            }
            statement => {
                return Err(RuntimeError::new(format!("Statement {} is not supported in an imperative context.", statement)))
            }
        };
        Ok(expression_id)
    }

    fn register_local(&mut self, identifier: &str, reference: Rc<ObjectReference>, scope: &mut scopes::Scope) {
        self.locals_names.insert(Rc::clone(&reference), identifier.to_string());
        scope.override_reference(scopes::Environment::Global, Reference::Local(reference), identifier);
    }

    fn link_expression_with_type(&mut self, syntax: &ast::Expression, type_declaration: &Option<ast::Expression>, scope: &scopes::Scope) -> RResult<ExpressionID> {
        let value = self.link_expression(syntax, scope)?;
        if let Some(type_declaration) = type_declaration {
            self.hint_type(value, type_declaration, scope)?
        }
        Ok(value)
    }

    pub fn link_expression(&mut self, syntax: &ast::Expression, scope: &scopes::Scope) -> RResult<ExpressionID> {
        let arguments: Vec<Positioned<precedence::Token>> = syntax.iter().map(|a| {
            self.link_term(a, scope)
                .err_in_range(&a.position)
        }).try_collect()?;

        link_patterns(arguments, scope, self)
    }

    pub fn link_term(&mut self, syntax: &Positioned<ast::Term>, scope: &scopes::Scope) -> RResult<Positioned<precedence::Token>> {
        let token = match &syntax.value {
            ast::Term::Error(err) => {
                return Err(vec![err.clone()])
            }
            ast::Term::Identifier(s) => {
                let variable = scope.resolve(scopes::Environment::Global, s)?;

                match variable {
                    Reference::Local(ref_) => {
                        let ObjectReference { id, type_, mutability } = ref_.as_ref();

                        precedence::Token::Expression(self.link_unambiguous_expression(
                            vec![],
                            type_,
                            ExpressionOperation::GetLocal(ref_.clone())
                        )?)
                    }
                    Reference::Keyword(keyword) => {
                        precedence::Token::Keyword(keyword.clone())
                    }
                    Reference::FunctionOverload(overload) => {
                        match overload.representation.form {
                            FunctionForm::GlobalFunction => {
                                precedence::Token::FunctionReference { overload: Rc::clone(overload), target: None }
                            }
                            FunctionForm::GlobalImplicit => {
                                precedence::Token::Expression(
                                    self.link_function_call(overload.functions.iter(), overload.representation.clone(), vec![], vec![], scope, syntax.position.clone())?
                                )
                            }
                            FunctionForm::MemberFunction => panic!(),
                            FunctionForm::MemberImplicit => panic!(),
                        }
                    }
                }
            }
            ast::Term::MacroIdentifier(s) => {
                todo!()
            }
            ast::Term::IntLiteral(string) => {
                let string_expression_id = self.link_string_literal(string)?;

                precedence::Token::Expression(self.link_abstract_function_call(
                    vec![string_expression_id],
                    Rc::clone(&self.runtime.traits.as_ref().unwrap().ConstructableByIntLiteral),
                    Rc::clone(&self.runtime.traits.as_ref().unwrap().parse_int_literal_function.target),
                    scope.traits.clone(),
                    syntax.position.clone(),
                )?)
            }
            ast::Term::RealLiteral(string) => {
                let string_expression_id = self.link_string_literal(string)?;

                precedence::Token::Expression(self.link_abstract_function_call(
                    vec![string_expression_id],
                    Rc::clone(&self.runtime.traits.as_ref().unwrap().ConstructableByRealLiteral),
                    Rc::clone(&self.runtime.traits.as_ref().unwrap().parse_real_literal_function.target),
                    scope.traits.clone(),
                    syntax.position.clone(),
                )?)
            }
            ast::Term::MemberAccess(access) => {
                let target = self.link_term(&access.target, scope)
                    .err_in_range(&syntax.position)?;

                guard!(let precedence::Token::Expression(target) = &target.value else {
                    return Err(RuntimeError::new(format!("Dot notation is not supported in this context.")))
                });

                let overload = scope.resolve(scopes::Environment::Member, &access.member)?
                    .as_function_overload()?;

                match overload.representation.form {
                    FunctionForm::MemberImplicit => {
                        precedence::Token::Expression(
                            self.link_function_call(overload.functions.iter(), overload.representation.clone(), vec![ParameterKey::Positional], vec![*target], scope, syntax.position.clone())?
                        )
                    }
                    FunctionForm::MemberFunction => {
                        precedence::Token::FunctionReference { overload, target: Some(*target) }
                    }
                    _ => unreachable!()
                }
            }
            ast::Term::Struct(s) => {
                precedence::Token::AnonymousStruct(self.link_struct(scope, s)?)
            }
            ast::Term::Array(a) => {
                let values = a.iter().map(|x| {
                    self.link_expression_with_type(&x.value, &x.type_declaration, scope)
                }).try_collect()?;

                precedence::Token::AnonymousArray {
                    keys: a.iter()
                        .map(|x| x.key.as_ref().try_map(|x| self.link_expression(x, scope)))
                        .try_collect()?,
                    values,
                }
            }
            ast::Term::StringLiteral(parts) => {
                precedence::Token::Expression(match &parts[..] {
                    // Simple case: Just one part means we can use it directly.
                    [] => self.link_string_part(
                        &syntax.with_value(ast::StringPart::Literal("".to_string())),
                        scope
                    )?,
                    [part] => self.link_string_part(part, scope)?,
                    _ => {
                        let mut parts: Vec<_> = parts.iter().map(|part| self.link_string_part(part, scope)).try_collect()?;
                        // TODO We should call concat() with an array instead.
                        let last = parts.pop().unwrap();
                        parts.into_iter().try_rfold(last, |rstring, lstring| {
                            // Call format(<args>)
                            self.link_simple_function_call("add", vec![ParameterKey::Positional, ParameterKey::Positional], vec![lstring, rstring], scope, syntax.position.clone())
                        })?
                    }
                })
            }
            ast::Term::Block(statements) => {
                precedence::Token::Expression(self.link_block(statements, &scope)?)
            }
        };

        Ok(syntax.with_value(token))
    }

    fn link_struct(&mut self, scope: &scopes::Scope, args: &Vec<ast::StructArgument>) -> RResult<Struct> {
        let values = args.iter().map(|x| {
            self.link_expression_with_type(&x.value, &x.type_declaration, scope)
        }).try_collect()?;

        Ok(Struct {
            keys: args.iter()
                .map(|x| x.key.clone())
                .collect(),
            values,
        })
    }

    pub fn link_string_part(&mut self, part: &Positioned<ast::StringPart>, scope: &scopes::Scope) -> RResult<ExpressionID> {
        match &part.value {
            ast::StringPart::Literal(literal) => {
                self.link_string_literal(literal)
            },
            ast::StringPart::Object(o) => {
                let struct_ = self.link_struct(scope, o)?;
                // Call format(<args>)
                self.link_simple_function_call("format", struct_.keys, struct_.values, scope, part.position.clone())
            }
        }
    }

    fn link_simple_function_call(&mut self, name: &str, keys: Vec<ParameterKey>, args: Vec<ExpressionID>, scope: &scopes::Scope, range: Range<usize>) -> RResult<ExpressionID> {
        let variable = scope.resolve(scopes::Environment::Global, name)?;

        match variable {
            Reference::FunctionOverload(overload) => {
                match overload.representation.form {
                    FunctionForm::GlobalFunction => {
                        let expression_id = self.link_function_call(overload.functions.iter(), overload.representation.clone(), keys, args, scope, range)?;
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

    pub fn link_conjunctive_pairs(&mut self, arguments: Vec<Positioned<ExpressionID>>, operations: Vec<Rc<FunctionHead>>) -> RResult<Positioned<ExpressionID>> {
        todo!()
    }

    pub fn link_function_call<'b>(&mut self, functions: impl Iterator<Item=&'b Rc<FunctionHead>>, representation: FunctionRepresentation, argument_keys: Vec<ParameterKey>, argument_expressions: Vec<ExpressionID>, scope: &scopes::Scope, range: Range<usize>) -> RResult<ExpressionID> {
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
                traits: scope.traits.clone(),
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
}
