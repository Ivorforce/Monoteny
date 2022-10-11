use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use uuid::Uuid;
use crate::linker::r#type::TypeFactory;
use crate::linker::{LinkError, scopes};
use crate::linker::scopes::Environment;
use crate::parser::abstract_syntax;
use crate::parser::abstract_syntax::PatternForm;
use crate::program::allocation::Reference;
use crate::program::functions::{FunctionForm, FunctionPointer, FunctionPointerTarget, HumanFunctionInterface, MachineFunctionInterface, ParameterKey};
use crate::program::generics::GenericAlias;
use crate::program::traits::{Trait, TraitConformanceDeclaration, TraitConformanceRequirement};
use crate::program::types::{TypeProto, TypeUnit};


pub fn link_function_pointer(function: &abstract_syntax::Function, scope: &scopes::Scope, requirements: &HashSet<Rc<TraitConformanceRequirement>>) -> Result<Rc<FunctionPointer>, LinkError> {
    let mut type_factory = TypeFactory::new(scope);

    let return_type = function.return_type.as_ref().map(|x| type_factory.link_type(&x)).unwrap_or_else(|| Ok(TypeProto::void()))?;

    let mut parameters: HashSet<Rc<Reference>> = HashSet::new();
    let mut parameter_names: Vec<(ParameterKey, Rc<Reference>)> = vec![];
    let mut parameter_names_internal: Vec<String> = vec![];

    if let Some(parameter) = &function.target {
        let variable = Reference::make_immutable(type_factory.link_type(&parameter.param_type)?);

        parameters.insert(Rc::clone(&variable));
        parameter_names.push((ParameterKey::Positional, variable));
        parameter_names_internal.push(parameter.internal_name.clone());
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
            alphanumeric_name: function.identifier.clone(),

            parameter_names,
            parameter_names_internal,

            form: if function.target.is_none() { FunctionForm::Global } else { FunctionForm::Member },
        }),
    }))
}

pub fn link_operator_pointer(function: &abstract_syntax::Operator, scope: &scopes::Scope, requirements: &HashSet<Rc<TraitConformanceRequirement>>) -> Result<Rc<FunctionPointer>, LinkError> {
    let mut type_factory = TypeFactory::new(scope);

    let return_type = function.return_type.as_ref().map(|x| type_factory.link_type(&x)).unwrap_or_else(|| Ok(TypeProto::void()))?;

    let mut parameters: HashSet<Rc<Reference>> = HashSet::new();
    let mut parameter_names: Vec<(ParameterKey, Rc<Reference>)> = vec![];
    let mut parameter_names_internal: Vec<String> = vec![];

    for parameter in function.lhs.iter().chain([&function.rhs]) {
        let variable = Reference::make_immutable(type_factory.link_type(&parameter.param_type)?);

        parameters.insert(Rc::clone(&variable));
        parameter_names.push((ParameterKey::Positional, variable));
        parameter_names_internal.push(parameter.internal_name.clone());
    }

    let pattern = scope.resolve_operator_pattern(
        &function.operator,
        &if function.lhs.is_some() { PatternForm::Binary } else { PatternForm::Unary }
    );

    Ok(Rc::new(FunctionPointer {
        pointer_id: Uuid::new_v4(),
        target: FunctionPointerTarget::Static { implementation_id: Uuid::new_v4() },

        machine_interface: Rc::new(MachineFunctionInterface {
            parameters,
            return_type,
            requirements: requirements.iter().chain(&type_factory.requirements).map(Rc::clone).collect(),
        }),
        human_interface: Rc::new(HumanFunctionInterface {
            name: function.operator.clone(),
            alphanumeric_name: pattern.alias.clone(),
            parameter_names,
            parameter_names_internal,

            form: FunctionForm::Pattern(pattern.precedence_group.form),
        }),
    }))
}