use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;
use guard::guard;
use itertools::Itertools;
use strum::IntoEnumIterator;
use try_map::FallibleMapExt;
use crate::interpreter::Runtime;
use crate::program::computation_tree::{ExpressionForest, ExpressionID, ExpressionOperation, Statement};
use crate::linker::{LinkError, precedence, scopes};
use crate::linker::ambiguous::{AmbiguousFunctionCall, AmbiguousFunctionCandidate, AmbiguousNumberLiteral, LinkerAmbiguity};
use crate::linker::precedence::link_patterns;
use crate::linker::r#type::TypeFactory;
use crate::linker::scopes::Scope;
use crate::parser::ast;
use crate::program::allocation::{ObjectReference, Reference};
use crate::program::functions::{FunctionForm, FunctionHead, FunctionOverload, FunctionPointer, ParameterKey};
use crate::program::generics::{GenericAlias, TypeForest};
use crate::program::global::FunctionImplementation;
use crate::program::r#struct::Struct;
use crate::program::traits::{RequirementsAssumption, TraitGraph};
use crate::program::types::*;

pub struct ImperativeLinker<'a> {
    pub function: Rc<FunctionHead>,

    pub runtime: &'a Runtime,

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

    pub fn link_function_body(mut self, body: &ast::Expression, scope: &scopes::Scope) -> Result<Box<FunctionImplementation>, LinkError> {
        let mut scope = scope.subscope();

        let granted_requirements = scope.traits.assume_granted(
            self.function.interface.requirements.iter()
                .map(|req| req.mapping_types(&|x| x.freezing_generics_to_any()))
        );

        // Let our scope know that our parameter types (all of type any!) conform to the requirements
        for conformance in granted_requirements.iter() {
            scope.traits.add_conformance(Rc::clone(conformance)).unwrap();
        };

        // Add abstract function mocks to our scope to be callable.
        for conformance in granted_requirements.iter() {
            for (abstract_function, function) in conformance.function_mapping.iter() {
                // TODO Do we need to keep track of the object reference created by this trait conformance?
                //  For the record, it SHOULD be created - an abstract function reference can still be passed around,
                //  assigned and maybe called later.
                let ptr = &conformance.binding.trait_.abstract_functions[abstract_function];
                scope.overload_function(
                    &Rc::new(FunctionPointer {
                        target: Rc::clone(function),
                        name: ptr.name.clone(),
                        form: ptr.form.clone(),
                    }),
                    &ObjectReference::new_immutable(TypeProto::unit(TypeUnit::Function(Rc::clone(&function))))
                )?;
            }
        }

        // TODO Register generic types as variables so they can be referenced in the function

        // Register parameters as variables.
        let mut parameter_variables = vec![];
        for parameter in self.function.interface.parameters.iter() {
            let parameter_variable = ObjectReference::new_immutable(parameter.type_.freezing_generics_to_any().clone());
            self.variable_names.insert(Rc::clone(&parameter_variable), parameter.internal_name.clone());
            scope.insert_singleton(
                scopes::Environment::Global,
                Reference::Object(Rc::clone(&parameter_variable)),
                &parameter.internal_name
            );
            parameter_variables.push(parameter_variable);
        }

        let statements: Vec<Box<Statement>> = self.link_top_scope(body, &scope)?;

        let mut has_changed = true;
        while !self.ambiguities.is_empty() {
            if !has_changed {
                // TODO Output which parts are ambiguous, and how, by asking the objects
                panic!("The function '{:?}' is ambiguous ({} times): \n{}\n\n", &self.function, self.ambiguities.len(), self.ambiguities.iter().map(|x| x.to_string()).join("\n"))
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

        Ok(Box::new(FunctionImplementation {
            implementation_id: self.function.function_id,
            head: self.function,
            requirements_assumption: Box::new(RequirementsAssumption { conformance: HashMap::from_iter(granted_requirements.into_iter().map(|c| (Rc::clone(&c.binding), c))) }),
            statements,
            expression_forest: self.expressions,
            type_forest: self.types,
            parameter_variables,
            variable_names: self.variable_names.clone(),
        }))
    }

    pub fn link_top_scope(&mut self, body: &ast::Expression, scope: &scopes::Scope) -> Result<Vec<Box<Statement>>, LinkError> {
        if let [term] = &body[..] {
            if let ast::Term::Scope(body ) = term.as_ref() {
                // Single-Scope function; for simplicity's sake we'll just collapse it here.
                return self.link_scope(body, &scope)
            }
        }

        // Single-Expression function; let's figure out if we need to infer a Return statement.

        let expression_id = self.link_expression(body, &scope)?;
        // Can be either void or a type, but either way should match the expression
        self.types.bind(expression_id, &self.function.interface.return_type.freezing_generics_to_any())?;

        if self.function.interface.return_type.unit.is_void() {
            return Ok(vec![Box::new(Statement::Expression(expression_id))])
        }
        else {
            return Ok(vec![Box::new(Statement::Return(Some(expression_id)))])
        }
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

    pub fn link_primitive(&mut self, value: &str, traits: TraitGraph, is_float: bool) -> Result<ExpressionID, LinkError> {
        let expression_id = self.register_new_expression(vec![]);
        self.register_ambiguity(Box::new(AmbiguousNumberLiteral {
            expression_id,
            value: value.to_string(),
            traits,
            is_float
        }))?;
        Ok(expression_id)
    }

    pub fn hint_type(&mut self, value: GenericAlias, type_declaration: &ast::Expression, scope: &scopes::Scope) -> Result<(), LinkError> {
        let mut type_factory = TypeFactory::new(&scope);

        let type_declaration = type_factory.link_type(&type_declaration)?;

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

    pub fn link_scope(&mut self, body: &Vec<Box<ast::Statement>>, scope: &scopes::Scope) -> Result<Vec<Box<Statement>>, LinkError> {
        let mut scope = scope.subscope();
        let mut statements: Vec<Box<Statement>> = Vec::new();

        for statement in body.iter() {
            match statement.as_ref() {
                ast::Statement::VariableDeclaration {
                    mutability, identifier, type_declaration, expression
                } => {
                    let new_value: ExpressionID = self.link_expression(&expression, &scope)?;

                    if let Some(type_declaration) = type_declaration {
                        self.hint_type(new_value, type_declaration, &scope);
                    }

                    let object_ref = Rc::new(ObjectReference { id: Uuid::new_v4(), type_: TypeProto::unit(TypeUnit::Generic(new_value)), mutability: mutability.clone() });
                    let variable = Reference::Object(Rc::clone(&object_ref));

                    statements.push(Box::new(
                        Statement::VariableAssignment(Rc::clone(&object_ref), new_value)
                    ));
                    self.variable_names.insert(object_ref, identifier.clone());
                    scope.override_reference(scopes::Environment::Global, variable, identifier);
                },
                ast::Statement::Return(expression) => {
                    if let Some(expression) = expression {
                        if self.function.interface.return_type.unit.is_void() {
                            panic!("Return statement offers a value when the function declares void.")
                        }

                        let result: ExpressionID = self.link_expression(expression, &scope)?;
                        self.types.bind(result, &self.function.interface.return_type.freezing_generics_to_any().as_ref())?;

                        statements.push(Box::new(Statement::Return(Some(result))));
                    }
                    else {
                        if !self.function.interface.return_type.unit.is_void() {
                            panic!("Return statement offers no value when the function declares an object.")
                        }

                        statements.push(Box::new(Statement::Return(None)));
                    }
                },
                ast::Statement::Expression(expression) => {
                    let expression: ExpressionID = self.link_expression(&expression, &scope)?;
                    statements.push(Box::new(Statement::Expression(expression)));
                }
                ast::Statement::VariableAssignment { variable_name, new_value } => {
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

    fn link_expression_with_type(&mut self, syntax: &ast::Expression, type_declaration: &Option<ast::Expression>, scope: &scopes::Scope) -> Result<ExpressionID, LinkError> {
        let value = self.link_expression(syntax, scope)?;
        if let Some(type_declaration) = type_declaration {
            self.hint_type(value, type_declaration, scope)?
        }
        Ok(value)
    }

    pub fn link_expression(&mut self, syntax: &ast::Expression, scope: &scopes::Scope) -> Result<ExpressionID, LinkError> {
        let arguments: Vec<precedence::Token> = syntax.iter().map(|a| {
            self.link_term(a, scope)
        }).try_collect()?;

        link_patterns(arguments, scope, self)
    }

    pub fn link_term(&mut self, syntax: &ast::Term, scope: &scopes::Scope) -> Result<precedence::Token, LinkError> {
        Ok(match syntax {
            ast::Term::Identifier(s) => {
                let variable = scope.resolve(scopes::Environment::Global, s)?;

                match variable {
                    Reference::Object(ref_) => {
                        let ObjectReference { id, type_, mutability } = ref_.as_ref();

                        precedence::Token::Expression(self.link_unambiguous_expression(
                            vec![],
                            type_,
                            ExpressionOperation::VariableLookup(ref_.clone())
                        )?)
                    }
                    Reference::Keyword(keyword) => {
                        precedence::Token::Keyword(keyword.clone())
                    }
                    Reference::FunctionOverload(overload) => {
                        match overload.form {
                            FunctionForm::Global => {
                                precedence::Token::FunctionReference { overload: Rc::clone(overload), target: None }
                            }
                            FunctionForm::Member => panic!(),
                            FunctionForm::Constant => {
                                precedence::Token::Expression(
                                    self.link_function_call(&overload.functions(), &overload.name, vec![], vec![], scope)?
                                )
                            }
                        }
                    }
                    Reference::PrecedenceGroup(_) => {
                        return Err(LinkError::LinkError { msg: format!("Precedence group references are not supported in expressions yet.") })
                    }
                }
            }
            ast::Term::Int(string) => {
                precedence::Token::Expression(self.link_primitive(
                    string,
                    scope.traits.clone(),
                    false,
                )?)
            }
            ast::Term::Float(string) => {
                precedence::Token::Expression(self.link_primitive(
                    string,
                    scope.traits.clone(),
                    true,
                )?)
            }
            ast::Term::MemberAccess { target, member_name } => {
                let target = self.link_term(target, scope)?;

                guard!(let precedence::Token::Expression(target) = target else {
                    return Err(LinkError::LinkError { msg: format!("Dot notation is not supported in this context.") })
                });

                let variable = scope.resolve(scopes::Environment::Member, member_name)?;

                if let Reference::FunctionOverload(overload) = variable {
                    precedence::Token::FunctionReference { overload: Rc::clone(overload), target: Some(target) }
                }
                else {
                    todo!("Member access is not supported yet!")
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
                    [part] => self.link_string_part(part, scope)?,
                    _ => {
                        let mut parts: Vec<_> = parts.iter().map(|part| self.link_string_part(part, scope)).try_collect()?;
                        // TODO We should call concat() with an array instead.
                        let last = parts.pop().unwrap();
                        parts.into_iter().try_rfold(last, |rstring, lstring| {
                            // Call format(<args>)
                            self.link_simple_function_call("add", vec![ParameterKey::Positional, ParameterKey::Positional], vec![lstring, rstring], scope)
                        })?
                    }
                })
            }
            ast::Term::Scope(statements) => {
                todo!("In-function scopes are not supported yet.")
            }
        })
    }

    fn link_struct(&mut self, scope: &scopes::Scope, args: &Vec<ast::StructArgument>) -> Result<Struct, LinkError> {
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

    pub fn link_string_part(&mut self, part: &ast::StringPart, scope: &scopes::Scope) -> Result<ExpressionID, LinkError> {
        match part {
            ast::StringPart::Literal(literal) => {
                self.link_unambiguous_expression(
                    vec![],
                    &TypeProto::unit(TypeUnit::Struct(Rc::clone(&self.runtime.builtins.core.traits.String))),
                    ExpressionOperation::StringLiteral(literal.clone())
                )
            }
            ast::StringPart::Object(o) => {
                let struct_ = self.link_struct(scope, o)?;
                // Call format(<args>)
                self.link_simple_function_call("format", struct_.keys, struct_.values, scope)
            }
        }
    }

    fn link_simple_function_call(&mut self, name: &str, keys: Vec<ParameterKey>, args: Vec<ExpressionID>, scope: &Scope) -> Result<ExpressionID, LinkError> {
        let variable = scope.resolve(scopes::Environment::Global, name)?;

        match variable {
            Reference::FunctionOverload(overload) => {
                match overload.form {
                    FunctionForm::Global => {
                        let expression_id = self.link_function_call(&overload.functions(), &overload.name, keys, args, scope)?;
                        // Make sure the return type is actually String.
                        self.types.bind(expression_id, &TypeProto::unit(TypeUnit::Struct(Rc::clone(&self.runtime.builtins.core.traits.String))))?;
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

    pub fn link_conjunctive_pairs(&mut self, arguments: Vec<ExpressionID>, operations: Vec<Rc<FunctionOverload>>) -> Result<ExpressionID, LinkError> {
        todo!()
    }

    pub fn link_function_call(&mut self, functions: &Vec<Rc<FunctionHead>>, fn_name: &str, argument_keys: Vec<ParameterKey>, argument_expressions: Vec<ExpressionID>, scope: &scopes::Scope) -> Result<ExpressionID, LinkError> {
        // TODO Check if any arguments are void before anything else
        let seed = Uuid::new_v4();

        let argument_keys: Vec<&ParameterKey> = argument_keys.iter().collect();

        let mut candidates_with_failed_signature = vec![];
        let mut candidates: Vec<Box<AmbiguousFunctionCandidate>> = vec![];

        for fun in functions.iter().map(Rc::clone) {
            let param_keys = fun.interface.parameters.iter().map(|x| &x.external_key).collect::<Vec<&ParameterKey>>();
            if param_keys != argument_keys {
                candidates_with_failed_signature.push(fun);
                continue;
            }

            candidates.push(Box::new(AmbiguousFunctionCandidate {
                param_types: fun.interface.parameters.iter()
                    .map(|x| x.type_.seeding_generics(&seed))
                    .collect(),
                return_type: fun.interface.return_type.seeding_generics(&seed),
                requirements: fun.interface.requirements.iter().map(|x| x.mapping_types(&|type_| type_.seeding_generics(&seed))).collect(),
                function: fun,
            }));
        }

        if candidates.len() >= 1 {
            let expression_id = self.register_new_expression(argument_expressions.clone());

            self.register_ambiguity(Box::new(AmbiguousFunctionCall {
                seed,
                expression_id,
                function_name: fn_name.to_string(),
                arguments: argument_expressions,
                traits: scope.traits.clone(),
                candidates,
                failed_candidates: vec![]
            }))?;

            return Ok(expression_id);
        }

        // TODO We should probably output the locations of candidates.

        if candidates_with_failed_signature.len() > 1 {
            panic!("function {}({:?}) could not be resolved. {} candidates have mismatching signatures.", fn_name, argument_keys.iter().join(", "), candidates_with_failed_signature.len())
        }

        if candidates_with_failed_signature.len() == 1 {
            // TODO Print passed arguments like a signature, not array
            let candidate = candidates_with_failed_signature.iter().next().unwrap();
            panic!("function {}({:?}) could not be resolved. Candidate has mismatching signature: {:?}", fn_name, argument_keys.iter().join(", "), candidate)
        }

        panic!("function {} could not be resolved.", fn_name)
    }
}
