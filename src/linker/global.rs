use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::rc::Rc;
use itertools::Itertools;
use uuid::Uuid;
use crate::error::{ErrInRange, RuntimeError};
use crate::interpreter::Runtime;
use crate::parser::ast;
use crate::program::computation_tree::*;
use crate::linker::imperative::ImperativeLinker;
use crate::linker::scopes;
use crate::linker::conformance::ConformanceLinker;
use crate::linker::interface::{link_function_pointer, link_operator_pointer};
use crate::linker::r#type::TypeFactory;
use crate::linker::scopes::Environment;
use crate::linker::traits::TraitLinker;
use crate::program::allocation::{ObjectReference, Reference};
use crate::program::traits::{Trait, TraitBinding, TraitConformance, TraitConformanceRule};
use crate::program::functions::{FunctionHead, FunctionType, FunctionForm, FunctionInterface, FunctionPointer, Parameter, ParameterKey};
use crate::program::generics::TypeForest;
use crate::program::global::BuiltinFunctionHint;
use crate::program::module::Module;
use crate::program::types::*;
use crate::util::position::Positioned;

pub struct GlobalLinker<'a> {
    pub runtime: &'a Runtime,
    pub global_variables: scopes::Scope<'a>,
    pub function_bodies: HashMap<Rc<FunctionHead>, Positioned<&'a ast::Expression>>,
    pub module: Module,
}

pub fn link_file(syntax: &ast::Module, scope: &scopes::Scope, runtime: &Runtime) -> Result<Box<Module>, Vec<RuntimeError>> {
    let mut global_linker = GlobalLinker {
        runtime,
        module: Module::new("main".to_string()),  // TODO Give it a name!
        global_variables: scope.subscope(),
        function_bodies: Default::default(),
    };

    // Resolve global types / interfaces
    for statement in &syntax.global_statements {
        global_linker.link_global_statement(statement, &HashSet::new())
            .err_in_range(&statement.position)
            .map_err(|e| vec![e])?;
    }

    let global_variable_scope = &global_linker.global_variables;

    // Resolve function bodies
    let mut errors = vec![];
    for (head, pbody) in global_linker.function_bodies.drain() {
        let mut variable_names = HashMap::new();

        // TODO Inject traits, not pointers
        let mut resolver = Box::new(ImperativeLinker {
            function: Rc::clone(&head),
            runtime,
            types: Box::new(TypeForest::new()),
            expressions: Box::new(ExpressionTree::new()),
            variable_names,
            ambiguities: vec![]
        });

        match resolver.link_function_body(&pbody.value, &global_variable_scope) {
            Ok(implementation) => {
                global_linker.module.fn_implementations.insert(Rc::clone(&head), implementation);
            }
            Err(e) => {
                errors.push(e.in_range(pbody.position.clone()));
            }
        }
    }

    match errors.is_empty() {
        true => Ok(Box::new(global_linker.module)),
        false => Err(errors)
    }
}

impl <'a> GlobalLinker<'a> {
    pub fn link_global_statement(&mut self, pstatement: &'a Positioned<ast::GlobalStatement>, requirements: &HashSet<Rc<TraitBinding>>) -> Result<(), RuntimeError> {
        match &pstatement.value {
            ast::GlobalStatement::Error(err) => {
                return Err(err.clone())
            }
            ast::GlobalStatement::Pattern(pattern) => {
                let pattern = self.link_pattern(pattern)?;
                self.module.patterns.insert(Rc::clone(&pattern));
                self.global_variables.add_pattern(pattern)?;
            }
            ast::GlobalStatement::FunctionDeclaration(syntax) => {
                let scope = &self.global_variables;
                let fun = link_function_pointer(&syntax, &scope, requirements)?;

                self.add_function(fun, &syntax.body, &syntax.decorators, pstatement.position.clone())?;
            }
            ast::GlobalStatement::Operator(syntax) => {
                let scope = &self.global_variables;
                let fun = link_operator_pointer(&syntax, &scope, requirements)?;

                self.add_function(fun, &syntax.body, &syntax.decorators, pstatement.position.clone())?;
            }
            ast::GlobalStatement::Trait(syntax) => {
                let mut trait_ = Trait::new_with_self(syntax.name.clone());

                let generic_self_type = trait_.create_generic_type("Self");
                // TODO module.add_trait also adds a reference; should we use the same?
                let generic_self_type_ref = Reference::Object(ObjectReference::new_immutable(TypeProto::meta(generic_self_type.clone())));

                let mut scope = self.global_variables.subscope();
                scope.insert_singleton(Environment::Global, generic_self_type_ref, "Self");

                let mut linker = TraitLinker {
                    trait_: &mut trait_,
                };
                for statement in syntax.statements.iter() {
                    linker.link_statement(&statement.value, requirements, &scope)
                        .err_in_range(&statement.position)?;
                }

                let trait_ = Rc::new(trait_);
                let meta_type_reference = self.module.add_trait(&trait_);

                if trait_.abstract_functions.is_empty() {
                    // Can be instantiated as a struct!

                    let struct_type = TypeProto::unit(TypeUnit::Struct(Rc::clone(&trait_)));
                    let conformance_binding = trait_.create_generic_binding(vec![("Self", struct_type.clone())]);

                    let conformance = TraitConformance::pure(conformance_binding.clone());
                    self.module.trait_conformance.add_conformance_rule(TraitConformanceRule::direct(Rc::clone(&conformance)));
                    self.global_variables.traits.add_conformance_rule(TraitConformanceRule::direct(conformance));

                    let new_function = Rc::new(FunctionPointer {
                        target: FunctionHead::new(
                            FunctionInterface::new_simple([TypeProto::meta(struct_type.clone())].into_iter(), struct_type),
                            FunctionType::Static
                        ),
                        name: "call_as_function".to_string(),
                        form: FunctionForm::Member,
                    });
                    self.module.add_function(&new_function);
                    self.module.fn_builtin_hints.insert(Rc::clone(&new_function.target), BuiltinFunctionHint::Constructor);
                    self.global_variables.overload_function(&new_function, &self.module.fn_references[&new_function.target])?;
                }

                self.global_variables.insert_singleton(
                    Environment::Global,
                    Reference::Object(meta_type_reference),
                    &trait_.name.clone()
                );
            }
            ast::GlobalStatement::Conformance(syntax) => {
                let mut type_factory = TypeFactory::new(&self.global_variables);
                let self_type = type_factory.link_type(&syntax.declared_for)?;
                let declared = self.global_variables
                    .resolve(Environment::Global, &syntax.declared)?
                    .as_trait()?;
                if declared.generics.keys().collect_vec() != vec!["Self"] {
                    // Requires 1) parsing generics that the programmer binds
                    // and  2) inserting new generics for each that isn't explicitly bound
                    panic!("Declaring traits with more than self generics is not supported yet")
                }

                let self_ref = Reference::Object(ObjectReference::new_immutable(TypeProto::meta(self_type.clone())));
                let self_binding = declared.create_generic_binding(vec![("Self", self_type)]);

                let mut scope = self.global_variables.subscope();
                scope.insert_singleton(Environment::Global, self_ref, "Self");

                let mut linker = ConformanceLinker { functions: vec![], };
                for statement in syntax.statements.iter() {
                    linker.link_statement(&statement.value, &requirements.union(&type_factory.requirements).cloned().collect(), &scope)
                        .err_in_range(&statement.position)?;
                }

                // TODO To be order independent, we should finalize after sorting...
                //  ... Or check inconsistencies only at the very end.
                let conformance = linker.finalize_conformance(self_binding, &type_factory.requirements)?;

                let rule = Rc::new(TraitConformanceRule {
                    generics: type_factory.generic_names(),
                    requirements: type_factory.requirements,
                    conformance,
                });
                self.module.trait_conformance.add_conformance_rule(rule.clone());
                self.global_variables.traits.add_conformance_rule(rule);

                for fun in linker.functions {
                    self.add_function(fun.pointer, fun.body, fun.decorators, pstatement.position.clone())?;
                }
            }
            ast::GlobalStatement::Macro(syntax) => {
                let fun = match syntax.macro_name.as_str() {
                    "main" => {
                        let fun = Rc::new(FunctionPointer {
                            target: FunctionHead::new(
                                Rc::new(FunctionInterface {
                                    parameters: vec![],
                                    return_type: TypeProto::unit(TypeUnit::Void),
                                    requirements: Default::default(),
                                }),
                                FunctionType::Static
                            ),
                            name: "main".to_string(),
                            form: FunctionForm::Global,
                        });
                        self.module.main_functions.push(Rc::clone(&fun.target));
                        fun
                    },
                    "transpile" => {
                        let fun = Rc::new(FunctionPointer {
                            target: FunctionHead::new(
                                Rc::new(FunctionInterface {
                                    parameters: vec![
                                        Parameter {
                                            external_key: ParameterKey::Positional,
                                            internal_name: String::from("transpiler"),
                                            type_: TypeProto::unit(TypeUnit::Struct(Rc::clone(&self.runtime.builtins.transpilation.Transpiler))),
                                        }
                                    ],
                                    return_type: TypeProto::unit(TypeUnit::Void),
                                    requirements: Default::default(),
                                }),
                                FunctionType::Static
                            ),
                            name: "transpile".to_string(),
                            form: FunctionForm::Global,
                        });
                        self.module.transpile_functions.push(Rc::clone(&fun.target));
                        fun
                    },
                    _ => return Err(RuntimeError::new(format!("Function macro could not be resolved: {}", syntax.macro_name))),
                };

                self.add_function(fun, &syntax.body, &syntax.decorators, pstatement.position.clone())?;
            }
        }

        Ok(())
    }

    pub fn link_pattern(&mut self, syntax: &ast::PatternDeclaration) -> Result<Rc<Pattern>, RuntimeError> {
        let precedence_group = self.global_variables.resolve_precedence_group(&syntax.precedence);

        Ok(Rc::new(Pattern {
            id: Uuid::new_v4(),
            alias: syntax.alias.clone(),
            precedence_group,
            parts: syntax.parts.clone(),
        }))
    }

    pub fn add_function(&mut self, pointer: Rc<FunctionPointer>, body: &'a Option<ast::Expression>, decorators: &Vec<String>, range: Range<usize>) -> Result<(), RuntimeError> {
        // Create a variable for the function
        self.module.add_function(&pointer);
        self.global_variables.overload_function(&pointer, &self.module.fn_references[&pointer.target])?;
        // if interface.is_member_function {
        // TODO Create an additional variable as Metatype.function(self, ...args)?
        // }

        if let Some(body) = body {
            self.function_bodies.insert(Rc::clone(&pointer.target), Positioned {
                value: body,
                position: range
            });
        }

        for decorator in decorators.iter() {
            return Err(RuntimeError::new(format!("Decorator could not be resolved: {}", decorator)))
        }

        Ok(())
    }
}
