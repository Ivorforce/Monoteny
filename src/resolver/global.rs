use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;

use itertools::Itertools;

use crate::ast;
use crate::error::{ErrInRange, RResult, RuntimeError};
use crate::interpreter::runtime::Runtime;
use crate::parser::expressions;
use crate::program::functions::{FunctionCallExplicity, FunctionHead, FunctionInterface, FunctionLogic, FunctionLogicDescriptor, FunctionRepresentation, FunctionTargetType};
use crate::program::module::Module;
use crate::program::traits::{Trait, TraitConformanceRule};
use crate::program::types::*;
use crate::resolver::conformance::ConformanceResolver;
use crate::resolver::decorations::try_parse_pattern;
use crate::resolver::function::resolve_function_body;
use crate::resolver::imports::resolve_imports;
use crate::resolver::interface::resolve_function_interface;
use crate::resolver::precedence_order::resolve_precedence_order;
use crate::resolver::traits::{try_make_struct, TraitResolver};
use crate::resolver::type_factory::TypeFactory;
use crate::resolver::{imports, referencible, scopes};
use crate::static_analysis;
use crate::util::position::Positioned;

pub struct GlobalResolver<'a> {
    pub runtime: &'a mut Runtime,
    pub global_variables: scopes::Scope<'a>,
    pub function_bodies: HashMap<Rc<FunctionHead>, Positioned<&'a ast::Expression>>,
    pub module: &'a mut Module,
}

pub fn resolve_file(syntax: &ast::Block, scope: &scopes::Scope, runtime: &mut Runtime, module: &mut Module) -> RResult<()> {
    let mut global_resolver = GlobalResolver {
        runtime,
        module,
        global_variables: scope.subscope(),
        function_bodies: Default::default(),
    };

    // Resolve global types / interfaces
    for statement in &syntax.statements {
        global_resolver.resolve_global_statement(statement)
            .err_in_range(&statement.value.position)?;
    }

    let global_variable_scope = global_resolver.global_variables;
    let runtime = global_resolver.runtime;

    // Resolve function bodies
    let mut errors = vec![];
    for (head, pbody) in global_resolver.function_bodies {
        match resolve_function_body(&head, pbody.value, &global_variable_scope, runtime).and_then(|mut imp| {
            static_analysis::check(&mut imp)?;
            Ok(imp)
        }) {
            Ok(implementation) => {
                runtime.source.fn_logic.insert(head, FunctionLogic::Implementation(implementation));
            }
            Err(e) => {
                errors.extend(e.iter().map(|e| e.clone().in_range(pbody.position.clone())));
            }
        }
    }

    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors)
    }
}

impl <'a> GlobalResolver<'a> {
    pub fn resolve_global_statement(&mut self, pstatement: &'a ast::Decorated<Positioned<ast::Statement>>) -> RResult<()> {
        match &pstatement.value.value {
            ast::Statement::FunctionDeclaration(syntax) => {
                let scope = &self.global_variables;
                let function_head = resolve_function_interface(&syntax.interface, &scope, Some(&mut self.module), &mut self.runtime, &Default::default(), &Default::default())?;

                for decoration in pstatement.decorations_as_vec()? {
                    let pattern = try_parse_pattern(decoration, Rc::clone(&function_head), &self.global_variables)?;
                    self.module.patterns.insert(Rc::clone(&pattern));
                    self.global_variables.grammar.add_pattern(pattern)?;
                }
                self.schedule_function_body(&function_head, syntax.body.as_ref(), pstatement.value.position.clone());
                self.add_function_interface(&function_head)?;
            }
            ast::Statement::Trait(syntax) => {
                pstatement.no_decorations()?;

                let mut trait_ = Trait::new_with_self(&syntax.name);

                trait_.add_simple_parent_requirement(&self.runtime.traits.as_ref().unwrap().Any);

                let generic_self_type = trait_.create_generic_type("Self");
                let generic_self_meta_type = TypeProto::one_arg(&self.runtime.Metatype, generic_self_type.clone());
                // This is not the same reference as what module.add_trait returns - that reference is for the global metatype getter.
                //  Inside, we use the Self getter.
                let generic_self_self_getter = FunctionHead::new_static(
                    vec![],
                    FunctionRepresentation::new("Self", FunctionTargetType::Global, FunctionCallExplicity::Implicit),
                    FunctionInterface::new_provider(&generic_self_meta_type, vec![]),
                );

                let mut scope = self.global_variables.subscope();
                scope.overload_function(&generic_self_self_getter, generic_self_self_getter.declared_representation.clone())?;
                self.runtime.source.trait_references.insert(Rc::clone(&generic_self_self_getter), Rc::clone(&trait_.generics["Self"]));

                let mut resolver = TraitResolver {
                    runtime: &mut self.runtime,
                    trait_: &mut trait_,
                    generic_self_type,
                };
                for statement in syntax.block.statements.iter() {
                    statement.no_decorations()?;

                    resolver.resolve_statement(&statement.value.value, &Default::default(), &Default::default(), &scope)
                        .err_in_range(&statement.value.position)?;
                }

                self.add_trait(&Rc::new(trait_))?;
            }
            ast::Statement::Conformance(syntax) => {
                pstatement.no_decorations()?;

                let mut type_factory = TypeFactory::new(&self.global_variables);
                let self_type = type_factory.resolve_type(&syntax.declared_for, true, &mut self.runtime)?;
                let declared_type = type_factory.resolve_type(&syntax.declared, false, &mut self.runtime)?;
                let TypeUnit::Struct(declared) = &declared_type.unit else {
                    panic!("Somehow, the resolved type wasn't a struct.")
                };
                if !declared_type.arguments.is_empty() {
                    return Err(RuntimeError::error("Conformance cannot be declared with bindings for now.").to_array());
                }

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
                    vec![],
                    FunctionRepresentation::new("Self", FunctionTargetType::Global, FunctionCallExplicity::Implicit),
                    FunctionInterface::new_provider(&self_meta_type, vec![])
                );
                let self_binding = declared.create_generic_binding(vec![("Self", self_type)]);

                let mut scope = self.global_variables.subscope();
                scope.overload_function(&self_getter, self_getter.declared_representation.clone())?;
                self.runtime.source.trait_references.insert(Rc::clone(&self_getter), self_trait);

                let mut resolver = ConformanceResolver { runtime: &mut self.runtime, functions: vec![], };
                for statement in syntax.block.statements.iter() {
                    statement.no_decorations()?;

                    resolver.resolve_statement(&statement.value.value, &&conformance_requirements, &generics.values().cloned().collect(), &scope)
                        .err_in_range(&statement.value.position)?;
                }

                // TODO To be order independent, we should finalize after sorting...
                //  ... Or check inconsistencies only at the very end.
                let conformance = resolver.finalize_conformance(self_binding, &conformance_requirements, &generics.values().cloned().collect())?;
                let functions = resolver.functions;

                self.module.add_conformance_rule(
                    Rc::new(TraitConformanceRule {
                        generics,
                        requirements: conformance_requirements,
                        conformance,
                    }),
                    &mut self.global_variables,
                );

                for fun in functions {
                    self.schedule_function_body(&fun.function, fun.body.as_ref(), pstatement.value.position.clone());
                    // TODO Instead of adding conformance functions statically, we should add the abstract function to the scope.
                    //  This will allow the compiler to determine "function exists but no declaration exists" in the future.
                    self.add_function_interface(&fun.function)?;
                }
            }
            ast::Statement::Expression(e) => {
                pstatement.no_decorations()?;
                e.no_errors()?;

                let parsed = expressions::parse(e, &self.global_variables.grammar)?;

                let expressions::Value::FunctionCall(target, call_struct) = &parsed.value else {
                    return Err(RuntimeError::error("Unexpected statement in global context.").to_array())
                };

                let expressions::Value::MacroIdentifier(macro_name) = &target.value else {
                    return Err(RuntimeError::error("Unexpected statement in global context.").to_array())
                };

                match macro_name.as_str() {
                    "precedence_order" => {
                        let precedence_order = resolve_precedence_order(call_struct, &self.global_variables)?;
                        self.module.precedence_order = Some(precedence_order.clone());
                        self.global_variables.grammar.set_precedence_order(precedence_order);
                        return Ok(())
                    }
                    "use" => {
                        for import in resolve_imports(call_struct, &self.global_variables)? {
                            self.import(&&import.relative_to(&self.module.name))?;
                        }
                        return Ok(())
                    }
                    "include" => {
                        for import in resolve_imports(call_struct, &self.global_variables)? {
                            let import = import.relative_to(&self.module.name);
                            self.import(&import)?;
                            self.module.included_modules.push(import);
                        }
                        return Ok(())
                    }
                    _ => return Err(
                        RuntimeError::error(format!("Unrecognized macro: {}!", macro_name).as_str()).to_array()
                    )
                }
            }
            _ => {
                return Err(
                    RuntimeError::error("Unexpected statement in global context.").to_array()
                )
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

    fn add_trait(&mut self, trait_: &Rc<Trait>) -> RResult<()> {
        referencible::add_trait(self.runtime, &mut self.module, Some(&mut self.global_variables), &trait_)?;
        try_make_struct(trait_, self)?;
        Ok(())
    }

    pub fn add_function_interface(&mut self, function: &Rc<FunctionHead>) -> RResult<()> {
        referencible::add_function(self.runtime, &mut self.module, Some(&mut self.global_variables), function)?;

        Ok(())
    }

    pub fn schedule_function_body(&mut self, head: &Rc<FunctionHead>, body: Option<&'a ast::Expression>, range: Range<usize>) {
        if let Some(body) = body {
            self.function_bodies.insert(Rc::clone(head), Positioned {
                value: body,
                position: range
            });
        }
        else {
            self.runtime.source.fn_logic.insert(Rc::clone(head), FunctionLogic::Descriptor(FunctionLogicDescriptor::Stub));
        }
    }
}
