use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use itertools::Itertools;

use crate::error::{RResult, RuntimeError};
use crate::interpreter::Runtime;
use crate::resolver::{fields, scopes};
use crate::resolver::global::GlobalResolver;
use crate::resolver::interface::resolve_function_interface;
use crate::resolver::type_factory::TypeFactory;
use crate::parser::ast;
use crate::program::allocation::{Mutability, ObjectReference};
use crate::program::function_object::{FunctionCallExplicity, FunctionRepresentation, FunctionTargetType};
use crate::program::functions::{FunctionHead, FunctionInterface, Parameter, ParameterKey};
use crate::program::global::{FunctionLogic, FunctionLogicDescriptor};
use crate::program::traits::{Trait, TraitBinding, TraitConformance, TraitConformanceRule};
use crate::program::types::TypeProto;
use crate::source::StructInfo;
use crate::util::fmt::fmta;

pub struct TraitResolver<'a> {
    pub runtime: &'a Runtime,
    pub trait_: &'a mut Trait,
    pub generic_self_type: Rc<TypeProto>,
}

impl <'a> TraitResolver<'a> {
    pub fn resolve_statement(&mut self, statement: &'a ast::Statement, requirements: &HashSet<Rc<TraitBinding>>, generics: &HashMap<String, Rc<Trait>>, scope: &scopes::Scope) -> RResult<()> {
        match statement {
            ast::Statement::FunctionDeclaration(syntax) => {
                let (fun, representation) = resolve_function_interface(&syntax.interface, &scope, None, &self.runtime, requirements, generics)?;
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

                let Some(type_declaration) = type_declaration else {
                    return Err(RuntimeError::new(format!("Trait variables must have explicit types.")));
                };

                let mut type_factory = TypeFactory::new(scope, &self.runtime);

                let variable_type = type_factory.resolve_type(type_declaration, true)?;

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

pub fn try_make_struct(trait_: &Rc<Trait>, resolver: &mut GlobalResolver) -> RResult<Option<Rc<StructInfo>>> {
    let mut unaccounted_for_abstract_functions: HashSet<_> = trait_.abstract_functions.keys().collect();
    trait_.field_hints.iter().for_each(|hint| {
        [&hint.getter, &hint.setter].into_iter().flatten().map(|g| unaccounted_for_abstract_functions.remove(g)).collect_vec();
    });

    if !unaccounted_for_abstract_functions.is_empty() {
        return Ok(None)
    }

    let mut field_names = HashMap::new();
    let mut field_getters = HashMap::new();
    let mut field_setters = HashMap::new();

    // Can be instantiated as a struct!

    let struct_type = TypeProto::unit_struct(trait_);
    let mut function_mapping = HashMap::new();
    let mut parameters = vec![
        Parameter {
            external_key: ParameterKey::Positional,
            internal_name: "type".to_string(),
            type_: TypeProto::one_arg(&resolver.runtime.Metatype, struct_type.clone()),
        }
    ];
    let mut fields = vec![];

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
            function_mapping.insert(Rc::clone(abstract_getter), Rc::clone(&struct_getter));
            field_getters.insert(Rc::clone(&variable_as_object), struct_getter);
        }
        if let Some(abstract_setter) = &abstract_field.setter {
            let struct_setter = struct_field.setter.clone().unwrap();
            function_mapping.insert(Rc::clone(abstract_setter), Rc::clone(&struct_setter));
            field_setters.insert(Rc::clone(&variable_as_object), struct_setter);
        }

        parameters.push(Parameter {
            external_key: ParameterKey::Name(abstract_field.name.clone()),
            internal_name: abstract_field.name.clone(),
            type_: abstract_field.type_.clone(),
        });
        field_names.insert(Rc::clone(&variable_as_object), abstract_field.name.clone());
        fields.push(variable_as_object);
    }

    let conformance = TraitConformance::new(
        trait_.create_generic_binding(vec![("Self", struct_type.clone())]),
        function_mapping,
    );
    let conformance_rule = TraitConformanceRule::direct(conformance);
    resolver.module.trait_conformance.add_conformance_rule(conformance_rule.clone());
    resolver.global_variables.trait_conformance.add_conformance_rule(conformance_rule);

    let struct_ = Rc::new(StructInfo {
        trait_: Rc::clone(trait_),
        constructor: FunctionHead::new_static(
            Rc::new(FunctionInterface {
                parameters,
                return_type: struct_type,
                requirements: Default::default(),
                generics: Default::default(),
            }),
        ),
        fields,
        field_names,
        field_getters,
        field_setters,
    });

    resolver.runtime.source.fn_logic.insert(
        Rc::clone(&struct_.constructor),
        FunctionLogic::Descriptor(FunctionLogicDescriptor::Constructor(Rc::clone(&struct_)))
    );
    resolver.add_function_interface(
        Rc::clone(&struct_.constructor),
        FunctionRepresentation::new("call_as_function", FunctionTargetType::Member, FunctionCallExplicity::Explicit),
        &vec![],
    )?;

    for (ref_, head) in struct_.field_getters.iter() {
        let name = &struct_.field_names[ref_];

        resolver.runtime.source.fn_logic.insert(
            Rc::clone(head),
            FunctionLogic::Descriptor(FunctionLogicDescriptor::GetMemberField(Rc::clone(&struct_), Rc::clone(ref_)))
        );
        resolver.add_function_interface(
            Rc::clone(head),
            FunctionRepresentation::new(name, FunctionTargetType::Member, FunctionCallExplicity::Implicit),
            &vec![])?
        ;
    }

    for (ref_, head) in struct_.field_setters.iter() {
        let name = &struct_.field_names[ref_];

        resolver.runtime.source.fn_logic.insert(
            Rc::clone(&head),
            FunctionLogic::Descriptor(FunctionLogicDescriptor::SetMemberField(Rc::clone(&struct_), Rc::clone(ref_)))
        );
        resolver.add_function_interface(
            Rc::clone(head),
            FunctionRepresentation::new(name, FunctionTargetType::Member, FunctionCallExplicity::Implicit),
            &vec![]
        )?;
    }

    Ok(Some(struct_))
}