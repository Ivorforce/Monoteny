use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use uuid::Uuid;
use crate::error::{ErrInRange, RResult, RuntimeError};
use crate::interpreter::Runtime;
use crate::parser::ast;
use crate::program::computation_tree::*;
use crate::linker::imperative::ImperativeLinker;
use crate::linker::scopes;
use crate::linker::conformance::ConformanceLinker;
use crate::linker::interface::{link_function_pointer, link_operator_pointer};
use crate::linker::precedence_order::link_precedence_order;
use crate::linker::type_factory::TypeFactory;
use crate::linker::traits::{TraitLinker, try_make_struct};
use crate::program::function_object::{FunctionForm, FunctionRepresentation};
use crate::program::traits::{Trait, TraitBinding, TraitConformanceRule};
use crate::program::functions::{FunctionHead, FunctionInterface, FunctionType, Parameter, ParameterKey};
use crate::program::generics::TypeForest;
use crate::program::module::Module;
use crate::program::types::*;
use crate::util::position::Positioned;

pub struct GlobalLinker<'a> {
    pub runtime: &'a mut Runtime,
    pub global_variables: scopes::Scope<'a>,
    pub function_bodies: HashMap<Rc<FunctionHead>, Positioned<&'a ast::Expression>>,
    pub module: Module,
}

pub fn link_file(syntax: &ast::Module, scope: &scopes::Scope, runtime: &mut Runtime) -> Result<Box<Module>, Vec<RuntimeError>> {
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

    let global_variable_scope = global_linker.global_variables;
    let runtime = global_linker.runtime;

    // Resolve function bodies
    let mut errors = vec![];
    for (head, pbody) in global_linker.function_bodies {
        let mut resolver = Box::new(ImperativeLinker {
            function: Rc::clone(&head),
            runtime,
            types: Box::new(TypeForest::new()),
            expressions: Box::new(ExpressionTree::new()),
            ambiguities: vec![],
            locals_names: Default::default(),
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
    pub fn link_global_statement(&mut self, pstatement: &'a Positioned<ast::Statement>, requirements: &HashSet<Rc<TraitBinding>>) -> RResult<()> {
        match &pstatement.value {
            ast::Statement::Pattern(pattern) => {
                let pattern = self.link_pattern(pattern)?;
                self.module.patterns.insert(Rc::clone(&pattern));
                self.global_variables.add_pattern(pattern)?;
            }
            ast::Statement::FunctionDeclaration(syntax) => {
                let scope = &self.global_variables;
                let (fun, representation) = link_function_pointer(&syntax, &scope, &self.runtime, requirements)?;

                if let Some(body) = &syntax.body {
                    self.schedule_function_body(Rc::clone(&fun), body, pstatement.position.clone());
                }
                self.add_function_interface(fun, representation, &syntax.decorators)?;
            }
            ast::Statement::Operator(syntax) => {
                let scope = &self.global_variables;
                let (fun, representation) = link_operator_pointer(&syntax, &scope, &self.runtime, requirements)?;

                if let Some(body) = &syntax.body {
                    self.schedule_function_body(Rc::clone(&fun), body, pstatement.position.clone());
                }
                self.add_function_interface(fun, representation, &syntax.decorators)?;
            }
            ast::Statement::Trait(syntax) => {
                let mut trait_ = Trait::new_with_self(syntax.name.clone());

                let generic_self_type = trait_.create_generic_type("Self");
                let generic_self_meta_type = TypeProto::meta(generic_self_type.clone());
                // This is not the same reference as what module.add_trait returns - that reference is for the global metatype getter.
                //  Inside, we use the Self getter.
                let generic_self_self_getter = FunctionHead::new_static(
                    FunctionInterface::new_provider(&generic_self_meta_type, vec![]),
                );

                let mut scope = self.global_variables.subscope();
                scope.overload_function(&generic_self_self_getter, FunctionRepresentation::new("Self", FunctionForm::GlobalImplicit))?;
                self.runtime.source.trait_references.insert(Rc::clone(&generic_self_self_getter), Rc::clone(&trait_.generics["Self"]));

                let mut linker = TraitLinker {
                    runtime: &self.runtime,
                    trait_: &mut trait_,
                    generic_self_type,
                };
                for statement in syntax.statements.iter() {
                    linker.link_statement(&statement.value, requirements, &scope)
                        .err_in_range(&statement.position)?;
                }

                self.add_trait(&Rc::new(trait_))?;
            }
            ast::Statement::Conformance(syntax) => {
                let mut type_factory = TypeFactory::new(&self.global_variables, &mut self.runtime);
                let self_type = type_factory.link_type(&syntax.declared_for, true)?;
                let self_meta_type = TypeProto::meta(self_type.clone());
                let declared = type_factory.resolve_trait(&syntax.declared)?;
                if declared.generics.keys().collect_vec() != vec!["Self"] {
                    // Requires 1) parsing generics that the programmer binds
                    // and  2) inserting new generics for each that isn't explicitly bound
                    panic!("Declaring traits with more than self generics is not supported yet")
                }
                let generics = type_factory.generics;
                let requirements = type_factory.requirements;

                // FIXME This is not ideal; technically the trait_references thing should be a BOUND trait,
                //  because the user may have bound some generics of self in the declaration.
                //  For now it's fine - determining the self type will be the task of the interpreter in the future anyway.
                let self_trait = match &self_type.unit {
                    TypeUnit::Struct(trait_) => Rc::clone(trait_),
                    _ => panic!()
                };

                let self_getter = FunctionHead::new_static(
                    FunctionInterface::new_provider(&self_meta_type, vec![]),
                );
                let self_binding = declared.create_generic_binding(vec![("Self", self_type)]);

                let mut scope = self.global_variables.subscope();
                scope.overload_function(&self_getter, FunctionRepresentation::new("Self", FunctionForm::GlobalImplicit))?;
                self.runtime.source.trait_references.insert(Rc::clone(&self_getter), self_trait);

                let mut linker = ConformanceLinker { runtime: &self.runtime, functions: vec![], };
                for statement in syntax.statements.iter() {
                    linker.link_statement(&statement.value, &requirements.union(&requirements).cloned().collect(), &scope)
                        .err_in_range(&statement.position)?;
                }

                // TODO To be order independent, we should finalize after sorting...
                //  ... Or check inconsistencies only at the very end.
                let conformance = linker.finalize_conformance(self_binding, &requirements)?;

                let rule = Rc::new(TraitConformanceRule {
                    generics,
                    requirements,
                    conformance,
                });
                self.module.trait_conformance.add_conformance_rule(rule.clone());
                self.global_variables.traits.add_conformance_rule(rule);

                for fun in linker.functions {
                    if let Some(body) = &fun.body {
                        self.schedule_function_body(Rc::clone(&fun.function), body, pstatement.position.clone());
                    }
                    self.add_function_interface(fun.function, fun.representation.clone(), fun.decorators)?;
                }
            }
            ast::Statement::Macro(syntax) => {
                let (fun, representation) = match syntax.macro_name.as_str() {
                    "main" => {
                        let (fun, representation) = (
                            FunctionHead::new_static(
                                Rc::new(FunctionInterface {
                                    parameters: vec![],
                                    return_type: TypeProto::unit(TypeUnit::Void),
                                    requirements: Default::default(),
                                    generics: Default::default(),
                                })
                            ),
                            FunctionRepresentation::new("main", FunctionForm::GlobalFunction)
                        );
                        self.module.main_functions.push(Rc::clone(&fun));
                        (fun, representation)
                    },
                    "transpile" => {
                        let transpiler_trait = self.runtime.source.module_by_name["transpilation"].trait_by_getter.values()
                            .filter(|x| x.name == "Transpiler")
                            .exactly_one().unwrap();

                        // TODO This should use a generic transpiler, not a struct.
                        let (fun, representation) = (
                            FunctionHead::new_static(
                                Rc::new(FunctionInterface {
                                    parameters: vec![
                                        Parameter {
                                            external_key: ParameterKey::Positional,
                                            internal_name: String::from("transpiler"),
                                            type_: TypeProto::unit(TypeUnit::Struct(Rc::clone(transpiler_trait))),
                                        }
                                    ],
                                    return_type: TypeProto::unit(TypeUnit::Void),
                                    requirements: Default::default(),
                                    generics: Default::default(),
                                }),
                            ),
                            FunctionRepresentation::new("transpile", FunctionForm::GlobalFunction)
                        );
                        self.module.transpile_functions.push(Rc::clone(&fun));
                        (fun, representation)
                    },
                    "precedence_order" => {
                        guard!(let Some(body) = &syntax.body else {
                            return Err(RuntimeError::new(format!("@{} needs an array literal body.", syntax.macro_name)))
                        });

                        let precedence_order = link_precedence_order(body)?;
                        self.module.precedence_order = Some(precedence_order.clone());
                        self.global_variables.set_precedence_order(precedence_order);
                        return Ok(())
                    }
                    _ => return Err(RuntimeError::new(format!("Function macro could not be resolved: {}", syntax.macro_name))),
                };

                if let Some(body) = &syntax.body {
                    self.schedule_function_body(Rc::clone(&fun), body, pstatement.position.clone());
                }
                self.add_function_interface(fun, representation, &syntax.decorators)?;
            }
            statement => {
                return Err(RuntimeError::new(format!("Statement {} is not supported in a global context.", statement)))
            }
        }

        Ok(())
    }

    pub fn link_pattern(&mut self, syntax: &ast::PatternDeclaration) -> RResult<Rc<Pattern>> {
        let precedence_group = self.global_variables.resolve_precedence_group(&syntax.precedence);

        Ok(Rc::new(Pattern {
            id: Uuid::new_v4(),
            alias: syntax.alias.clone(),
            precedence_group,
            parts: syntax.parts.clone(),
        }))
    }

    fn add_trait(&mut self, trait_: &Rc<Trait>) -> Result<(), RuntimeError> {
        let getter = self.module.add_trait(&trait_);
        self.global_variables.overload_function(&getter, FunctionRepresentation::new(&trait_.name, FunctionForm::GlobalImplicit))?;
        self.runtime.source.trait_references.insert(getter, Rc::clone(trait_));

        try_make_struct(trait_, self)?;
        Ok(())
    }

    pub fn add_function_interface(&mut self, pointer: Rc<FunctionHead>, representation: FunctionRepresentation, decorators: &Vec<String>) -> RResult<()> {
        // Add the function to our module
        self.module.add_function(Rc::clone(&pointer), representation.clone());

        // Make it usable in our current scope.
        self.global_variables.overload_function(&pointer, representation)?;
        // The runtime also needs to know about the function.
        // TODO Technically we need to load it too, because future linked functions may want to call it.
        self.runtime.source.fn_getters.insert(
            Rc::clone(&pointer),
            Rc::clone(&self.module.fn_getters[&pointer])
        );

        // if interface.is_member_function {
        // TODO Create an additional variable as Metatype.function(self, ...args)?
        // }

        for decorator in decorators.iter() {
            return Err(RuntimeError::new(format!("Decorator could not be resolved: {}", decorator)))
        }

        Ok(())
    }

    pub fn schedule_function_body(&mut self, head: Rc<FunctionHead>, body: &'a ast::Expression, range: Range<usize>) {
        self.function_bodies.insert(Rc::clone(&head), Positioned {
            value: body,
            position: range
        });
    }
}
