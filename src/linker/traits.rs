use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use crate::error::{RResult, RuntimeError};
use crate::interpreter::Runtime;
use crate::linker::global::GlobalLinker;
use crate::linker::{fields, scopes};
use crate::linker::interface::link_function_interface;
use crate::linker::type_factory::TypeFactory;
use crate::parser::ast;
use crate::program::allocation::{Mutability, ObjectReference};
use crate::program::function_object::{FunctionForm, FunctionRepresentation};
use crate::program::functions::{FunctionHead, FunctionInterface, Parameter, ParameterKey};
use crate::program::global::FunctionLogicDescriptor;
use crate::program::traits::{Trait, TraitBinding, TraitConformance, TraitConformanceRule};
use crate::program::types::TypeProto;
use crate::util::fmt::fmta;

pub struct TraitLinker<'a> {
    pub runtime: &'a Runtime,
    pub trait_: &'a mut Trait,
    pub generic_self_type: Box<TypeProto>,
}

impl <'a> TraitLinker<'a> {
    pub fn link_statement(&mut self, statement: &'a ast::Statement, requirements: &HashSet<Rc<TraitBinding>>, generics: &HashMap<String, Rc<Trait>>, scope: &scopes::Scope) -> RResult<()> {
        match statement {
            ast::Statement::FunctionDeclaration(syntax) => {
                let (fun, representation) = link_function_interface(&syntax.interface, &scope, None, &self.runtime, requirements, generics)?;
                if !syntax.body.is_none() {
                    return Err(RuntimeError::new(format!("Abstract function {} cannot have a body.", fmta(|fmt| fun.format(fmt, &representation)))));
                };

                self.trait_.insert_function(fun, representation);
            }
            ast::Statement::VariableDeclaration { mutability, identifier, type_declaration, assignment } => {
                if let Some(_) = assignment {
                    return Err(RuntimeError::new(format!("Trait variables cannot have defaults until default monads are supported.")));
                }
                if !requirements.is_empty() {
                    return Err(RuntimeError::new(format!("Trait variables cannot have requirements.")));
                }

                guard!(let Some(type_declaration) = type_declaration else {
                    return Err(RuntimeError::new(format!("Trait variables must have explicit types.")));
                });

                let mut type_factory = TypeFactory::new(scope, &self.runtime);

                let variable_type = type_factory.link_type(type_declaration, true)?;

                if TypeProto::contains_generics([&variable_type].into_iter()) {
                    return Err(RuntimeError::new(format!("Variables cannot be generic: {}", identifier)));
                }

                let field = fields::make(
                    identifier,
                    &self.generic_self_type,
                    &variable_type,
                    true,
                    mutability == &Mutability::Mutable,
                );
                fields::add_to_trait(&mut self.trait_, field);
            }
            ast::Statement::Expression(e) => {
                e.no_errors()?;
                return Err(RuntimeError::new(format!("Expression {} not valid in a trait context.", statement)));
            }
            _ => {
                return Err(RuntimeError::new(format!("Statement {} not valid in a trait context.", statement)));
            }
        }

        Ok(())
    }
}

pub fn try_make_struct(trait_: &Rc<Trait>, linker: &mut GlobalLinker) -> RResult<()> {
    let mut unaccounted_for_abstract_functions: HashSet<_> = trait_.abstract_functions.keys().collect();
    trait_.field_hints.iter().for_each(|hint| {
        [&hint.getter, &hint.setter].into_iter().flatten().map(|g| unaccounted_for_abstract_functions.remove(g)).collect_vec();
    });

    if !unaccounted_for_abstract_functions.is_empty() {
        return Ok(())
    }

    // Can be instantiated as a struct!

    let struct_type = TypeProto::unit_struct(trait_);
    let mut function_mapping = HashMap::new();
    let mut parameters = vec![
        Parameter {
            external_key: ParameterKey::Positional,
            internal_name: "type".to_string(),
            type_: TypeProto::one_arg(&linker.runtime.Metatype, struct_type.clone()),
        }
    ];
    let mut parameter_mapping = vec![];

    for abstract_field in trait_.field_hints.iter() {
        let variable_as_object = ObjectReference::new_immutable(abstract_field.type_.clone());
        let struct_field = fields::make(
            &abstract_field.name,
            &struct_type,
            &abstract_field.type_,
            abstract_field.getter.is_some(),
            abstract_field.setter.is_some(),
        );

        // TODO Once generic types are supported, the variable type should be mapped to actual types
        if let Some(abstract_getter) = &abstract_field.getter {
            let struct_getter = struct_field.getter.clone().unwrap();
            linker.runtime.source.fn_logic_descriptors.insert(
                Rc::clone(&struct_getter),
                FunctionLogicDescriptor::GetMemberField(Rc::clone(trait_), Rc::clone(&variable_as_object))
            );
            function_mapping.insert(Rc::clone(abstract_getter), Rc::clone(&struct_getter));
            linker.add_function_interface(
                struct_getter,
                FunctionRepresentation::new(&struct_field.name, FunctionForm::MemberImplicit),
                &vec![])?
            ;
        }
        if let Some(abstract_setter) = &abstract_field.setter {
            let struct_setter = struct_field.setter.clone().unwrap();
            linker.runtime.source.fn_logic_descriptors.insert(
                Rc::clone(&struct_setter),
                FunctionLogicDescriptor::SetMemberField(Rc::clone(trait_), Rc::clone(&variable_as_object))
            );
            function_mapping.insert(Rc::clone(abstract_setter), Rc::clone(&struct_setter));
            linker.add_function_interface(
                struct_setter,
                FunctionRepresentation::new(&struct_field.name, FunctionForm::MemberImplicit),
                &vec![]
            )?;
        }

        parameters.push(Parameter {
            external_key: ParameterKey::Name(abstract_field.name.clone()),
            internal_name: abstract_field.name.clone(),
            type_: abstract_field.type_.clone(),
        });
        parameter_mapping.push(variable_as_object);
    }

    let conformance = TraitConformance::new(
        trait_.create_generic_binding(vec![("Self", struct_type.clone())]),
        function_mapping,
    );
    let conformance_rule = TraitConformanceRule::direct(conformance);
    linker.module.trait_conformance.add_conformance_rule(conformance_rule.clone());
    linker.global_variables.trait_conformance.add_conformance_rule(conformance_rule);

    let constructor = FunctionHead::new_static(
        Rc::new(FunctionInterface {
            parameters,
            return_type: struct_type,
            requirements: Default::default(),
            generics: Default::default(),
        }),
    );
    linker.runtime.source.fn_logic_descriptors.insert(
        Rc::clone(&constructor),
        FunctionLogicDescriptor::Constructor(Rc::clone(trait_), parameter_mapping)
    );
    linker.add_function_interface(
        constructor,
        FunctionRepresentation::new("call_as_function", FunctionForm::MemberFunction),
        &vec![],
    )?;

    Ok(())
}
