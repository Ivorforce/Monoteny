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
use crate::program::allocation::{ObjectReference, Reference};
use crate::program::functions::{FunctionForm, FunctionPointer, FunctionCallType, FunctionInterface, ParameterKey, Parameter, Function};
use crate::program::generics::GenericAlias;
use crate::program::traits::{Trait, TraitConformanceDeclaration, TraitConformanceRequirement};
use crate::program::types::{PatternPart, TypeProto, TypeUnit};


pub fn link_function_pointer(function: &abstract_syntax::Function, scope: &scopes::Scope, requirements: &HashSet<Rc<TraitConformanceRequirement>>) -> Result<Rc<FunctionPointer>, LinkError> {
    let mut type_factory = TypeFactory::new(scope);

    let return_type = function.return_type.as_ref().map(|x| type_factory.link_type(&x)).unwrap_or_else(|| Ok(TypeProto::void()))?;

    let mut parameters: Vec<Parameter> = vec![];

    if let Some(parameter) = &function.target_type {
        let variable = ObjectReference::new_immutable(type_factory.link_type(parameter)?);

        parameters.push(Parameter {
            external_key: ParameterKey::Positional,
            internal_name: String::from("self"),
            target: Rc::clone(&variable),
        });
    }

    for parameter in function.parameters.iter() {
        let variable = ObjectReference::new_immutable(type_factory.link_type(&parameter.param_type)?);

        parameters.push(Parameter {
            external_key: parameter.key.clone(),
            internal_name: parameter.internal_name.clone(),
            target: Rc::clone(&variable),
        });
    }

    Ok(Rc::new(FunctionPointer {
        pointer_id: Uuid::new_v4(),
        call_type: FunctionCallType::Static,
        target: Function::new(Rc::new(FunctionInterface {
            parameters,
            return_type,
            requirements: requirements.iter().chain(&type_factory.requirements).map(Rc::clone).collect(),
        })),
        name: function.identifier.clone(),
        form: if function.target_type.is_none() { FunctionForm::Global } else { FunctionForm::Member },
    }))
}

pub fn link_operator_pointer(function: &abstract_syntax::OperatorFunction, scope: &scopes::Scope, requirements: &HashSet<Rc<TraitConformanceRequirement>>) -> Result<Rc<FunctionPointer>, LinkError> {
    let mut type_factory = TypeFactory::new(scope);

    let return_type = function.return_type.as_ref().map(|x| type_factory.link_type(&x)).unwrap_or_else(|| Ok(TypeProto::void()))?;

    if let [OperatorArgument::Keyword(name)] = &function.parts.iter().map(|x| x.as_ref()).collect_vec()[..] {
        // Constant

        let fun = Rc::new(FunctionPointer {
            pointer_id: Uuid::new_v4(),
            call_type: FunctionCallType::Static,
            target: Function::new(Rc::new(FunctionInterface {
                parameters: vec![],
                return_type,
                requirements: requirements.iter().chain(&type_factory.requirements).map(Rc::clone).collect(),
            })),
            name: name.clone(),
            form: FunctionForm::Constant,
        });

        return Ok(fun)
    }

    // TODO Throw if multiple patterns match
    for pattern in scope.patterns.iter() {
        guard!(let Some(arguments) = match_patterns(&pattern.parts, &function.parts) else {
            continue;
        });

        let mut parameters: Vec<Parameter> = vec![];

        for (key, internal_name, type_expression) in arguments.into_iter() {
            let variable = ObjectReference::new_immutable(type_factory.link_type(type_expression)?);

            parameters.push(Parameter {
                external_key: key.clone(),
                internal_name: internal_name.clone(),
                target: Rc::clone(&variable),
            });
        }

        return Ok(Rc::new(FunctionPointer {
            pointer_id: Uuid::new_v4(),
            call_type: FunctionCallType::Static,
            target: Function::new(Rc::new(FunctionInterface {
                parameters,
                return_type,
                requirements: requirements.iter().chain(&type_factory.requirements).map(Rc::clone).collect(),
            })),
            name: pattern.alias.clone(),
            form: FunctionForm::Global,
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