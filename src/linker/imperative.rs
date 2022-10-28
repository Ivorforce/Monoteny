use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use uuid::Uuid;
use guard::guard;
use itertools::{Itertools, zip_eq};
use strum::IntoEnumIterator;
use try_map::{FallibleMapExt, FlipResultExt};
use crate::linker;
use crate::program::computation_tree::{ExpressionForest, ExpressionID, ExpressionOperation, Statement};
use crate::linker::{LinkError, precedence, scopes};
use crate::linker::ambiguous::{AmbiguousFunctionCall, AmbiguousFunctionCandidate, AmbiguousNumberPrimitive, LinkerAmbiguity};
use crate::linker::precedence::{link_patterns};
use crate::linker::r#type::TypeFactory;
use crate::parser::abstract_syntax;
use crate::program::allocation::{Mutability, ObjectReference, Reference, ReferenceType};
use crate::program::builtins::Builtins;
use crate::program::functions::{FunctionForm, FunctionOverload, FunctionPointer, FunctionPointerTarget, HumanFunctionInterface, MachineFunctionInterface, ParameterKey};
use crate::program::generics::{GenericAlias, TypeForest};
use crate::program::global::FunctionImplementation;
use crate::program::primitives;
use crate::program::traits::{Trait, TraitBinding, TraitConformanceDeclaration, TraitConformanceRequirement, TraitConformanceScope};
use crate::program::types::*;

pub struct ImperativeLinker<'a> {
    pub function: Rc<FunctionPointer>,

    pub builtins: &'a Builtins,

    pub types: Box<TypeForest>,
    pub expressions: Box<ExpressionForest>,
    pub ambiguities: Vec<Box<dyn LinkerAmbiguity>>,

    pub variable_names: HashMap<Rc<ObjectReference>, String>,
}

impl <'a> ImperativeLinker<'a> {
    pub fn register_new_expression(&mut self, arguments: Vec<ExpressionID>) -> ExpressionID {
        let id = ExpressionID::new_v4();

        self.types.register(id);
        self.expressions.arguments.insert(id, arguments);

        id
    }

    pub fn link_function_body(mut self, body: &Vec<Box<abstract_syntax::Statement>>, scope: &scopes::Scope) -> Result<Rc<FunctionImplementation>, LinkError> {
        let mut conformance_delegations = HashMap::new();
        let mut scope = scope.subscope();
        for requirement in self.function.machine_interface.requirements.iter() {
            let declaration = Trait::assume_granted(&requirement.trait_, requirement.arguments.clone());
            scope.add_trait_conformance(&declaration);
            conformance_delegations.insert(Rc::clone(requirement), declaration);
        }

        // TODO Register generics as variables so they can be referenced in the function

        for (internal_name, (_, variable)) in zip_eq(self.function.human_interface.parameter_names_internal.iter(), self.function.human_interface.parameter_names.iter()) {
            scope.insert_singleton(scopes::Environment::Global, Reference::make(ReferenceType::Object(variable.clone())), internal_name);
        }

        let statements: Vec<Box<Statement>> = self.link_top_scope(body, &scope)?;

        let mut has_changed = true;
        while !self.ambiguities.is_empty() {
            if !has_changed {
                // TODO Output which parts are ambiguous, and how, by asking the objects
                panic!("The function {} is ambiguous.", &self.function.human_interface.name)
            }

            has_changed = false;

            let callbacks: Vec<Box<dyn LinkerAmbiguity>> = self.ambiguities.drain(..).collect();
            for mut ambiguity in callbacks {
                if ambiguity.attempt_to_resolve(&mut self)? {
                    has_changed = true;
                }
                else {
                    self.ambiguities.push(ambiguity);
                }
            }
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
            type_forest: self.types,
            variable_names: self.variable_names.clone(),
            conformance_delegations
        }))
    }

    pub fn link_top_scope(&mut self, body: &Vec<Box<abstract_syntax::Statement>>, scope: &scopes::Scope) -> Result<Vec<Box<Statement>>, LinkError> {
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
        let id = self.register_new_expression(arguments);

        self.expressions.operations.insert(id.clone(), operation);

        self.types.bind(id, &return_type)
            .map(|_| id)
    }

    pub fn register_ambiguity(&mut self, mut ambiguity: Box<dyn LinkerAmbiguity>) -> Result<(), LinkError> {
        match ambiguity.attempt_to_resolve(self) {
            Ok(true) => Ok(()),  // Done already!
            Ok(false) => {
                 self.ambiguities.push(ambiguity);
                Ok(())  // Need to resolve later.
            },
            // We errored already!
            Err(err) => Err(err)
        }
    }

    pub fn link_primitive(&mut self, value: &String, traits: TraitConformanceScope, is_float: bool) -> Result<ExpressionID, LinkError> {
        let expression_id = self.register_new_expression(vec![]);
        self.register_ambiguity(Box::new(AmbiguousNumberPrimitive {
            expression_id,
            value: value.clone(),
            traits,
            is_float
        }))?;
        Ok(expression_id)
    }

    pub fn link_scope(&mut self, body: &Vec<Box<abstract_syntax::Statement>>, scope: &scopes::Scope) -> Result<Vec<Box<Statement>>, LinkError> {
        let mut scope = scope.subscope();
        let mut statements: Vec<Box<Statement>> = Vec::new();

        for statement in body.iter() {
            match statement.as_ref() {
                abstract_syntax::Statement::VariableDeclaration {
                    mutability, identifier, type_declaration, expression
                } => {
                    let new_value: ExpressionID = self.link_expression(&expression, &scope)?;

                    if let Some(type_declaration) = type_declaration {
                        let mut type_factory = TypeFactory::new(&scope);

                        let type_declaration = type_factory.link_type(&type_declaration)?;

                        for requirement in type_factory.requirements {
                            todo!("Implicit imperative requirements are not implemented yet")
                        }

                        for (name, generic) in type_factory.generics.into_iter() {
                            scope.insert_singleton(
                                scopes::Environment::Global,
                                Reference::make_immutable_type(TypeProto::unit(generic.clone())),
                                &name
                            );
                        }

                        self.types.bind(new_value, type_declaration.as_ref())?;
                    }

                    let object_ref = Rc::new(ObjectReference { id: Uuid::new_v4(), type_: TypeProto::unit(TypeUnit::Generic(new_value)), mutability: mutability.clone() });
                    let variable = Rc::new(Reference {
                        id: Uuid::new_v4(),
                        type_: ReferenceType::Object(Rc::clone(&object_ref)),
                    });

                    statements.push(Box::new(
                        Statement::VariableAssignment(Rc::clone(&object_ref), new_value)
                    ));
                    self.variable_names.insert(object_ref, identifier.clone());
                    scope.override_variable(scopes::Environment::Global, variable, identifier);
                },
                abstract_syntax::Statement::Return(expression) => {
                    if let Some(expression) = expression {
                        if self.function.machine_interface.return_type.unit.is_void() {
                            panic!("Return statement offers a value when the function declares void.")
                        }

                        let result: ExpressionID = self.link_expression(expression.as_ref(), &scope)?;

                        self.types.bind(result, &self.function.machine_interface.return_type.as_ref())?;
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
                    let expression: ExpressionID = self.link_expression(&expression, &scope)?;
                    statements.push(Box::new(Statement::Expression(expression)));
                }
                abstract_syntax::Statement::VariableAssignment { variable_name, new_value } => {
                    let ref_ = scope.resolve(scopes::Environment::Global, variable_name)?
                        .as_object_ref(true)?;

                    let new_value: ExpressionID = self.link_expression(&new_value, &scope)?;
                    self.types.bind(new_value, &ref_.type_)?;

                    statements.push(Box::new(
                        Statement::VariableAssignment(Rc::clone(&ref_), new_value)
                    ));
                }
            }
        }

        Ok(statements)
    }

    pub fn link_expression(&mut self, syntax: &abstract_syntax::Expression, scope: &scopes::Scope) -> Result<ExpressionID, LinkError> {
        let arguments: Vec<precedence::Token> = syntax.iter().map(|a| {
            self.link_term(a, scope)
        }).try_collect()?;

        link_patterns(arguments, scope, self)
    }

    pub fn link_term(&mut self, syntax: &abstract_syntax::Term, scope: &scopes::Scope) -> Result<precedence::Token, LinkError> {
        Ok(match syntax {
            abstract_syntax::Term::Identifier(s) => {
                if s == "true" || s == "false" {
                    // TODO Once we have constants, register these as constants instead.
                    //  Yes, that makes them shadowable. Sue me.
                    precedence::Token::Expression(self.link_unambiguous_expression(
                        vec![],
                        &TypeProto::unit(TypeUnit::Primitive(primitives::Type::Bool)),
                        ExpressionOperation::BoolLiteral(s == "true")
                    )?)
                }
                else {
                    let variable = scope.resolve(scopes::Environment::Global, s)?;

                    match &variable.type_ {
                        ReferenceType::Object(ref_) => {
                            let ObjectReference { id, type_, mutability } = ref_.as_ref();

                            precedence::Token::Expression(self.link_unambiguous_expression(
                                vec![],
                                type_,
                                ExpressionOperation::VariableLookup(ref_.clone())
                            )?)
                        }
                        ReferenceType::Keyword(keyword) => {
                            precedence::Token::Keyword(keyword.clone())
                        }
                        ReferenceType::FunctionOverload(overload) => {
                            match overload.form {
                                FunctionForm::Global => {
                                    precedence::Token::FunctionReference { overload: Rc::clone(overload), target: None }
                                }
                                FunctionForm::Member => panic!(),
                                FunctionForm::Constant => {
                                    precedence::Token::Expression(self.link_function_call(&overload.pointers, &overload.name, vec![], vec![], scope)?)
                                }
                            }
                        }
                        ReferenceType::PrecedenceGroup(_) => {
                            return Err(LinkError::LinkError { msg: format!("Precedence group references are not supported in expressions yet.") })
                        }
                        ReferenceType::Trait(_) => {
                            return Err(LinkError::LinkError { msg: format!("Trait references are not supported in expressions yet.") })
                        }
                    }
                }
            }
            abstract_syntax::Term::Int(string) => {
                precedence::Token::Expression(self.link_primitive(
                    string,
                    scope.trait_conformance_declarations.clone(),
                    false,
                )?)
            }
            abstract_syntax::Term::Float(string) => {
                precedence::Token::Expression(self.link_primitive(
                    string,
                    scope.trait_conformance_declarations.clone(),
                    true,
                )?)
            }
            abstract_syntax::Term::MemberAccess { target, member_name } => {
                let target = self.link_term(target, scope)?;

                guard!(let precedence::Token::Expression(target) = target else {
                    return Err(LinkError::LinkError { msg: format!("Dot notation is not supported in this context.") })
                });

                let variable = scope.resolve(scopes::Environment::Global, member_name)?;

                if let ReferenceType::FunctionOverload(overload) = &variable.type_ {
                    precedence::Token::FunctionReference { overload: Rc::clone(overload), target: Some(target) }
                }
                else {
                    todo!("Member access is not supported yet!")
                }
            }
            abstract_syntax::Term::Struct(s) => {
                precedence::Token::AnonymousStruct {
                    keys: s.iter().map(|x| x.key.clone()).collect(),
                    values: s.iter().map(|x| self.link_expression(&x.value, scope)).try_collect()?,
                }
            }
            abstract_syntax::Term::Array(a) => {
                precedence::Token::AnonymousArray {
                    keys: a.iter().map(|x| x.key.as_ref().try_map(|x| self.link_expression(x, scope))).try_collect()?,
                    values: a.iter().map(|x| self.link_expression(&x.value, scope)).try_collect()?,
                }
            }
            abstract_syntax::Term::StringLiteral(string) => {
                precedence::Token::Expression(self.link_unambiguous_expression(
                    vec![],
                    &TypeProto::unit(TypeUnit::Struct(Rc::clone(&self.builtins.traits.String))),
                    ExpressionOperation::StringLiteral(string.clone())
                )?)
            }
            abstract_syntax::Term::TypeHint { object, type_ } => {
                todo!()
            }
        })
    }

    pub fn link_conjunctive_pairs(&mut self, arguments: Vec<ExpressionID>, operations: Vec<Rc<FunctionOverload>>) -> Result<ExpressionID, LinkError> {
        todo!()
    }

    pub fn link_function_call(&mut self, functions: &HashSet<Rc<FunctionPointer>>, fn_name: &String, argument_keys: Vec<ParameterKey>, argument_expressions: Vec<ExpressionID>, scope: &scopes::Scope) -> Result<ExpressionID, LinkError> {
        // TODO Check if any arguments are void before anything else
        let seed = Uuid::new_v4();

        let argument_keys: Vec<&ParameterKey> = argument_keys.iter().collect();

        let mut candidates_with_failed_signature = vec![];
        let mut candidates: Vec<Box<AmbiguousFunctionCandidate>> = vec![];

        for fun in functions.iter().map(Rc::clone) {
            let param_keys = fun.human_interface.parameter_names.iter().map(|x| &x.0).collect::<Vec<&ParameterKey>>();
            if param_keys != argument_keys {
                candidates_with_failed_signature.push(fun);
                continue;
            }

            candidates.push(Box::new(AmbiguousFunctionCandidate {
                param_types: fun.human_interface.parameter_names.iter()
                    .map(|x| x.1.type_.with_any_as_generic(&seed))
                    .collect(),
                return_type: fun.machine_interface.return_type.with_any_as_generic(&seed),
                requirements: fun.machine_interface.requirements.iter().map(|x| x.with_any_as_generic(&seed)).collect(),
                function: fun,
            }));
        }

        if candidates.len() >= 1 {
            let expression_id = self.register_new_expression(argument_expressions.clone());

            self.register_ambiguity(Box::new(AmbiguousFunctionCall {
                expression_id,
                function_name: fn_name.clone(),
                arguments: argument_expressions,
                trait_conformance_declarations: scope.trait_conformance_declarations.clone(),
                candidates,
                failed_candidates: vec![]
            }))?;

            return Ok(expression_id);
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
