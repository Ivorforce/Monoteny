use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use itertools::Itertools;

use crate::ast;
use crate::error::{RResult, RuntimeError};
use crate::interpreter::builtins::traits::make_any_functions;
use crate::interpreter::runtime::Runtime;
use crate::program::allocation::{Mutability, ObjectReference};
use crate::program::functions::{FunctionHead, FunctionInterface, FunctionLogic, FunctionLogicDescriptor, FunctionRepresentation, Parameter, ParameterKey};
use crate::program::traits::{StructInfo, Trait, TraitBinding, TraitConformance, TraitConformanceRule};
use crate::program::types::TypeProto;
use crate::resolver::{fields, scopes};
use crate::resolver::global::GlobalResolver;
use crate::resolver::interface::resolve_function_interface;
use crate::resolver::type_factory::TypeFactory;

pub struct TraitResolver<'a> {
    pub runtime: &'a Runtime,
    pub trait_: &'a mut Trait,
    pub generic_self_type: Rc<TypeProto>,
}

impl <'a> TraitResolver<'a> {
    pub fn resolve_statement(&mut self, statement: &'a ast::Statement, requirements: &HashSet<Rc<TraitBinding>>, generics: &HashSet<Rc<Trait>>, scope: &scopes::Scope) -> RResult<()> {
        match statement {
            ast::Statement::FunctionDeclaration(syntax) => {
                // TODO What do we do with the parameter names? They don't belong in the interface. Probably the runtime source?
                let function_head = resolve_function_interface(&syntax.interface, &scope, None, &self.runtime, requirements, generics)?;
                if !syntax.body.is_none() {
                    return Err(
                        RuntimeError::error(format!("Abstract function {:?} cannot have a body.", function_head).as_str()).to_array()
                    );
                };

                self.trait_.abstract_functions.insert(function_head);
            }
            ast::Statement::VariableDeclaration { mutability, identifier, type_declaration, assignment } => {
                if let Some(_) = assignment {
                    return Err(
                        RuntimeError::error("Trait variables cannot have defaults until default monads are supported.").to_array()
                    );
                }
                if !requirements.is_empty() {
                    return Err(
                        RuntimeError::error("Trait variables cannot have requirements.").to_array()
                    );
                }

                let Some(type_declaration) = type_declaration else {
                    return Err(
                        RuntimeError::error("Trait variables must have explicit types.").to_array()
                    );
                };

                let mut type_factory = TypeFactory::new(scope, &self.runtime);

                let variable_type = type_factory.resolve_type(type_declaration, true)?;

                if TypeProto::contains_generics([&variable_type].into_iter()) {
                    return Err(
                        RuntimeError::error(format!("Variables cannot be generic: {}", identifier).as_str()).to_array()
                    );
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
            _ => {
                if let ast::Statement::Expression(exp) = statement {
                    // It may still just be an error!
                    exp.no_errors()?;
                }
                return Err(
                    RuntimeError::error("Statement not valid in a trait context.").to_array()
                );
            }
        }

        Ok(())
    }
}

pub fn try_make_struct(trait_: &Rc<Trait>, resolver: &mut GlobalResolver) -> RResult<Option<Rc<StructInfo>>> {
    let mut unaccounted_for_abstract_functions = trait_.abstract_functions.clone();
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
            type_: abstract_field.type_.clone(),
        });
        field_names.insert(Rc::clone(&variable_as_object), abstract_field.name.clone());
        fields.push(variable_as_object);
    }

    resolver.module.add_conformance_rule(
        TraitConformanceRule::direct(TraitConformance::new(
            trait_.create_generic_binding(vec![("Self", struct_type.clone())]),
            function_mapping,
        )),
        &mut resolver.global_variables
    );

    let any_functions = make_any_functions(&struct_type);
    resolver.runtime.source.fn_logic.insert(
        Rc::clone(&any_functions.clone),
        FunctionLogic::Descriptor(FunctionLogicDescriptor::Clone(struct_type.clone()))
    );
    resolver.add_function_interface(&any_functions.clone)?;

    let traits = resolver.runtime.traits.as_ref().unwrap();

    resolver.module.add_conformance_rule(
        TraitConformanceRule::manual(
            traits.Any.create_generic_binding(vec![("Self", struct_type.clone())]),
            vec![
                (&traits.Any_functions.clone, &any_functions.clone),
            ]
        ),
        &mut resolver.global_variables
    );

    let constructor = FunctionHead::new_static(
        FunctionHead::infer_param_names(&parameters),
        FunctionRepresentation::new_member_function("call_as_function"),
        Rc::new(FunctionInterface {
            parameters,
            return_type: struct_type,
            requirements: Default::default(),
            generics: Default::default(),
        }),
    );

    let struct_ = Rc::new(StructInfo {
        trait_: Rc::clone(trait_),
        clone: any_functions.clone.clone(),
        constructor: Rc::clone(&constructor),
        fields,
        field_names,
        field_getters,
        field_setters,
    });

    resolver.runtime.source.fn_logic.insert(
        Rc::clone(&constructor),
        FunctionLogic::Descriptor(FunctionLogicDescriptor::Constructor(Rc::clone(&struct_)))
    );
    resolver.add_function_interface(&constructor)?;

    for (ref_, head) in struct_.field_getters.iter() {
        let name = &struct_.field_names[ref_];

        resolver.runtime.source.fn_logic.insert(
            Rc::clone(head),
            FunctionLogic::Descriptor(FunctionLogicDescriptor::GetMemberField(Rc::clone(&struct_), Rc::clone(ref_)))
        );
        resolver.add_function_interface(head)?;
    }

    for (ref_, head) in struct_.field_setters.iter() {
        let name = &struct_.field_names[ref_];

        resolver.runtime.source.fn_logic.insert(
            Rc::clone(&head),
            FunctionLogic::Descriptor(FunctionLogicDescriptor::SetMemberField(Rc::clone(&struct_), Rc::clone(ref_)))
        );
        resolver.add_function_interface(head )?;
    }

    Ok(Some(struct_))
}
