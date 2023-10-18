use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::rc::Rc;
use itertools::Itertools;
use uuid::Uuid;
use crate::error::{ErrInRange, RResult, RuntimeError};
use crate::interpreter::Runtime;
use crate::parser::ast;
use crate::program::computation_tree::*;
use crate::linker::imperative::ImperativeLinker;
use crate::linker::{imports, interpreter_mock, referencible, scopes};
use crate::linker::conformance::ConformanceLinker;
use crate::linker::imports::link_imports;
use crate::linker::interface::{link_function_interface, link_operator_interface};
use crate::linker::precedence_order::link_precedence_order;
use crate::linker::type_factory::TypeFactory;
use crate::linker::traits::{TraitLinker, try_make_struct};
use crate::program::function_object::{FunctionForm, FunctionRepresentation};
use crate::program::traits::{Trait, TraitBinding, TraitConformanceRule};
use crate::program::functions::{FunctionHead, FunctionInterface};
use crate::program::generics::TypeForest;
use crate::program::module::{Module, ModuleName};
use crate::program::types::*;
use crate::util::position::Positioned;

pub struct GlobalLinker<'a> {
    pub runtime: &'a mut Runtime,
    pub global_variables: scopes::Scope<'a>,
    pub function_bodies: HashMap<Rc<FunctionHead>, Positioned<&'a ast::Expression>>,
    pub module: Module,
}

pub fn link_file(syntax: &ast::Module, scope: &scopes::Scope, runtime: &mut Runtime, name: ModuleName) -> RResult<Box<Module>> {
    let mut global_linker = GlobalLinker {
        runtime,
        module: Module::new(name),  // TODO Give it a name!
        global_variables: scope.subscope(),
        function_bodies: Default::default(),
    };

    // Resolve global types / interfaces
    for statement in &syntax.global_statements {
        global_linker.link_global_statement(statement, &HashSet::new())
            .err_in_range(&statement.position)?;
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
                runtime.source.fn_implementations.insert(Rc::clone(&head), implementation);
            }
            Err(e) => {
                errors.extend(e.iter().map(|e| e.in_range(pbody.position.clone())));
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
                let (fun, representation) = link_function_interface(&syntax.interface, &scope, Some(&mut self.module), &self.runtime, requirements, &HashMap::new())?;

                if let Some(body) = &syntax.body {
                    self.schedule_function_body(Rc::clone(&fun), body, pstatement.position.clone());
                }
                self.add_function_interface(fun, representation, &vec![])?;
            }
            ast::Statement::Operator(syntax) => {
                let scope = &self.global_variables;
                let (fun, representation) = link_operator_interface(&syntax, &scope, &self.runtime, requirements)?;

                if let Some(body) = &syntax.body {
                    self.schedule_function_body(Rc::clone(&fun), body, pstatement.position.clone());
                }
                self.add_function_interface(fun, representation, &vec![])?;
            }
            ast::Statement::Trait(syntax) => {
                let mut trait_ = Trait::new_with_self(&syntax.name);

                let generic_self_type = trait_.create_generic_type("Self");
                let generic_self_meta_type = TypeProto::one_arg(&self.runtime.Metatype, generic_self_type.clone());
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
                    linker.link_statement(&statement.value, requirements, &HashMap::new(), &scope)
                        .err_in_range(&statement.position)?;
                }

                self.add_trait(&Rc::new(trait_))?;
            }
            ast::Statement::Conformance(syntax) => {
                let mut type_factory = TypeFactory::new(&self.global_variables, &mut self.runtime);
                let self_type = type_factory.link_type(&syntax.declared_for, true)?;
                let declared = type_factory.resolve_trait(&syntax.declared)?;
                if declared.generics.keys().collect_vec() != vec!["Self"] {
                    // Requires 1) parsing generics that the programmer binds
                    // and  2) inserting new generics for each that isn't explicitly bound
                    panic!("Declaring traits with more than self generics is not supported yet")
                }
                let generics = type_factory.generics;
                let conformance_requirements = type_factory.requirements;

                // FIXME This is not ideal; technically the trait_references thing should be a BOUND trait,
                //  because the user may have bound some generics of self in the declaration.
                //  For now it's fine - determining the self type will be the task of the interpreter in the future anyway.
                let self_trait = match &self_type.unit {
                    TypeUnit::Struct(trait_) => Rc::clone(trait_),
                    _ => panic!()
                };

                let self_meta_type = TypeProto::one_arg(&self.runtime.Metatype, self_type.clone());
                let self_getter = FunctionHead::new_static(
                    FunctionInterface::new_provider(&self_meta_type, vec![]),
                );
                let self_binding = declared.create_generic_binding(vec![("Self", self_type)]);

                let mut scope = self.global_variables.subscope();
                scope.overload_function(&self_getter, FunctionRepresentation::new("Self", FunctionForm::GlobalImplicit))?;
                self.runtime.source.trait_references.insert(Rc::clone(&self_getter), self_trait);

                let mut linker = ConformanceLinker { runtime: &self.runtime, functions: vec![], };
                for statement in syntax.statements.iter() {
                    linker.link_statement(&statement.value, &requirements.union(&conformance_requirements).cloned().collect(), &generics, &scope)
                        .err_in_range(&statement.position)?;
                }

                // TODO To be order independent, we should finalize after sorting...
                //  ... Or check inconsistencies only at the very end.
                let conformance = linker.finalize_conformance(self_binding, &conformance_requirements, &generics)?;

                let rule = Rc::new(TraitConformanceRule {
                    generics,
                    requirements: conformance_requirements,
                    conformance,
                });
                self.module.trait_conformance.add_conformance_rule(rule.clone());
                self.global_variables.traits.add_conformance_rule(rule);

                for fun in linker.functions {
                    if let Some(body) = &fun.body {
                        self.schedule_function_body(Rc::clone(&fun.function), body, pstatement.position.clone());
                    }
                    self.add_function_interface(fun.function, fun.representation.clone(), &fun.decorators)?;
                }
            }
            ast::Statement::Expression(e) => {
                e.no_errors()?;

                match &e[..] {
                    [l, r] => {
                        match (&l.value, &r.value) {
                            (ast::Term::MacroIdentifier(macro_name), ast::Term::Struct(args)) => {
                                match macro_name.as_str() {
                                    "precedence_order" => {
                                        let body = interpreter_mock::plain_parameter(format!("{}!", macro_name).as_str(), args)?;

                                        let precedence_order = link_precedence_order(body)?;
                                        self.module.precedence_order = Some(precedence_order.clone());
                                        self.global_variables.set_precedence_order(precedence_order);
                                        return Ok(())
                                    }
                                    "use" => {
                                        for import in link_imports(args)? {
                                            self.import(&&import.relative_to(&self.module.name))?;
                                        }
                                        return Ok(())
                                    }
                                    "include" => {
                                        for import in link_imports(args)? {
                                            let import = import.relative_to(&self.module.name);
                                            self.import(&import)?;
                                            self.module.included_modules.push(import);
                                        }
                                        return Ok(())
                                    }
                                    _ => return Err(RuntimeError::new(format!("Unrecognized macro: {}!", macro_name)))
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {},
                }

                return Err(RuntimeError::new(format!("Expression {} is not supported in a global context.", e)))
            }
            statement => {
                return Err(RuntimeError::new(format!("Statement {} is not supported in a global context.", statement)))
            }
        }

        Ok(())
    }

    fn import(&mut self, import: &Vec<String>) -> RResult<()> {
        let root_module = self.runtime.get_or_load_module(import)?;
        let root_module_name = root_module.name.clone();
        imports::deep(&mut self.runtime, root_module_name, &mut self.global_variables)?;
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

    fn add_trait(&mut self, trait_: &Rc<Trait>) -> RResult<()> {
        referencible::add_trait(self.runtime, &mut self.module, Some(&mut self.global_variables), &trait_)?;
        try_make_struct(trait_, self)?;
        Ok(())
    }

    pub fn add_function_interface(&mut self, pointer: Rc<FunctionHead>, representation: FunctionRepresentation, decorators: &Vec<String>) -> RResult<()> {
        for decorator in decorators.iter() {
            return Err(RuntimeError::new(format!("Decorator could not be resolved: {}", decorator)))
        }

        referencible::add_function(self.runtime, &mut self.module, Some(&mut self.global_variables), pointer, representation)?;

        Ok(())
    }

    pub fn schedule_function_body(&mut self, head: Rc<FunctionHead>, body: &'a ast::Expression, range: Range<usize>) {
        self.function_bodies.insert(Rc::clone(&head), Positioned {
            value: body,
            position: range
        });
    }
}
