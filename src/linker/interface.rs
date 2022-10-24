use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use guard::guard;
use itertools::{Itertools, zip_eq};
use uuid::Uuid;
use crate::linker::r#type::TypeFactory;
use crate::linker::{LinkError, scopes};
use crate::linker::scopes::Environment;
use crate::parser::abstract_syntax;
use crate::parser::abstract_syntax::{Expression, OperatorArgument};
use crate::program::allocation::Reference;
use crate::program::functions::{FunctionForm, FunctionPointer, FunctionPointerTarget, HumanFunctionInterface, MachineFunctionInterface, ParameterKey};
use crate::program::generics::GenericAlias;
use crate::program::traits::{Trait, TraitConformanceDeclaration, TraitConformanceRequirement};
use crate::program::types::{PatternPart, TypeProto, TypeUnit};


pub fn link_function_pointer(function: &abstract_syntax::Function, scope: &scopes::Scope, requirements: &HashSet<Rc<TraitConformanceRequirement>>) -> Result<Rc<FunctionPointer>, LinkError> {
    let mut type_factory = TypeFactory::new(scope);

    let return_type = function.return_type.as_ref().map(|x| type_factory.link_type(&x)).unwrap_or_else(|| Ok(TypeProto::void()))?;

    let mut parameters: HashSet<Rc<Reference>> = HashSet::new();
    let mut parameter_names: Vec<(ParameterKey, Rc<Reference>)> = vec![];
    let mut parameter_names_internal: Vec<String> = vec![];

    if let Some(parameter) = &function.target_type {
        let variable = Reference::make_immutable(type_factory.link_type(parameter)?);

        parameters.insert(Rc::clone(&variable));
        parameter_names.push((ParameterKey::Positional, variable));
        parameter_names_internal.push(String::from("self"));
    }

    for parameter in function.parameters.iter() {
        let variable = Reference::make_immutable(type_factory.link_type(&parameter.param_type)?);

        parameters.insert(Rc::clone(&variable));
        parameter_names.push((parameter.key.clone(), variable));
        parameter_names_internal.push(parameter.internal_name.clone());
    }

    Ok(Rc::new(FunctionPointer {
        pointer_id: Uuid::new_v4(),
        target: FunctionPointerTarget::Static { implementation_id: Uuid::new_v4() },

        machine_interface: Rc::new(MachineFunctionInterface {
            parameters,
            return_type,
            requirements: requirements.iter().chain(&type_factory.requirements).map(Rc::clone).collect(),
        }),
        human_interface: Rc::new(HumanFunctionInterface {
            name: function.identifier.clone(),

            parameter_names,
            parameter_names_internal,

            form: if function.target_type.is_none() { FunctionForm::Global } else { FunctionForm::Member },
        }),
    }))
}

pub fn link_operator_pointer(function: &abstract_syntax::OperatorFunction, scope: &scopes::Scope, requirements: &HashSet<Rc<TraitConformanceRequirement>>) -> Result<Rc<FunctionPointer>, LinkError> {
    let mut type_factory = TypeFactory::new(scope);

    let return_type = function.return_type.as_ref().map(|x| type_factory.link_type(&x)).unwrap_or_else(|| Ok(TypeProto::void()))?;

    // TODO Throw if multiple patterns match
    for pattern in scope.patterns.iter() {
        guard!(let Some(arguments) = match_patterns(&pattern.parts, &function.parts) else {
            continue;
        });

        let mut parameters: HashSet<Rc<Reference>> = HashSet::new();
        let mut parameter_names: Vec<(ParameterKey, Rc<Reference>)> = vec![];
        let mut parameter_names_internal: Vec<String> = vec![];

        for (key, internal_name, type_expression) in arguments.into_iter() {
            let variable = Reference::make_immutable(type_factory.link_type(type_expression)?);

            parameters.insert(Rc::clone(&variable));
            parameter_names.push((key.clone(), variable));
            parameter_names_internal.push(internal_name.clone());
        }

        return Ok(Rc::new(FunctionPointer {
            pointer_id: Uuid::new_v4(),
            target: FunctionPointerTarget::Static { implementation_id: Uuid::new_v4() },

            machine_interface: Rc::new(MachineFunctionInterface {
                parameters,
                return_type,
                requirements: requirements.iter().chain(&type_factory.requirements).map(Rc::clone).collect(),
            }),
            human_interface: Rc::new(HumanFunctionInterface {
                name: pattern.alias.clone(),
                parameter_names,
                parameter_names_internal,

                form: FunctionForm::Global,
            }),
        }))
    }

    return Err(LinkError::LinkError { msg: String::from("Unknown pattern in function definition.") });
}

pub fn match_patterns<'a>(pattern_parts: &'a Vec<Box<PatternPart>>, function_parts: &'a Vec<Box<OperatorArgument>>) -> Option<Vec<(&'a ParameterKey, &'a String, &'a Expression)>> {
    if pattern_parts.len() != function_parts.len() {
        return None;
    }

    let mut parameters: Vec<(&ParameterKey, &String, &Expression)> = vec![];

    for (pattern_part, function_part) in zip_eq(pattern_parts.iter(), function_parts.iter()) {
        match (pattern_part.as_ref(), function_part.as_ref()) {
            (PatternPart::Keyword(k1), OperatorArgument::Keyword(k2)) => {
                if k1 != k2 {
                    return None;
                }
            }
            (PatternPart::Parameter { key, internal_name }, OperatorArgument::Parameter(expression)) => {
                parameters.push((key, internal_name, expression));
            }
            _ => return None,
        }
    }

    Some(parameters)
}