use std::collections::HashSet;
use std::rc::Rc;
use guard::guard;
use itertools::{Itertools, zip_eq};
use crate::error::{RResult, RuntimeError};
use crate::interpreter::Runtime;
use crate::linker::type_factory::TypeFactory;
use crate::linker::scopes;
use crate::parser::ast;
use crate::parser::ast::{Expression, OperatorArgument};
use crate::program::functions::{FunctionForm, FunctionPointer, FunctionType, FunctionInterface, ParameterKey, Parameter, FunctionHead};
use crate::program::traits::TraitBinding;
use crate::program::types::{PatternPart, TypeProto};


pub fn link_function_pointer(function: &ast::Function, scope: &scopes::Scope, runtime: &Runtime, requirements: &HashSet<Rc<TraitBinding>>) -> RResult<Rc<FunctionPointer>> {
    let mut type_factory = TypeFactory::new(scope, runtime);

    let return_type = function.return_type.as_ref().map(|x| type_factory.link_type(&x, true)).unwrap_or_else(|| Ok(TypeProto::void()))?;

    let mut parameters: Vec<Parameter> = vec![];

    if let Some(parameter) = &function.target_type {
        parameters.push(Parameter {
            external_key: ParameterKey::Positional,
            internal_name: String::from("self"),
            type_: type_factory.link_type(parameter, true)?,
        });
    }

    for parameter in function.parameters.iter() {
        parameters.push(Parameter {
            external_key: parameter.key.clone(),
            internal_name: parameter.internal_name.clone(),
            type_: type_factory.link_type(&parameter.param_type, true)?,
        });
    }

    Ok(Rc::new(FunctionPointer {
        target: FunctionHead::new(
            Rc::new(FunctionInterface {
                parameters,
                return_type,
                requirements: requirements.iter().chain(&type_factory.requirements).map(Rc::clone).collect(),
                generics: type_factory.generics,
            }),
            FunctionType::Static
        ),
        name: function.identifier.clone(),
        form: if function.target_type.is_none() { FunctionForm::GlobalFunction } else { FunctionForm::MemberFunction },
    }))
}

pub fn link_operator_pointer(function: &ast::OperatorFunction, scope: &scopes::Scope, runtime: &Runtime, requirements: &HashSet<Rc<TraitBinding>>) -> RResult<Rc<FunctionPointer>> {
    let mut type_factory = TypeFactory::new(scope, runtime);

    let return_type = function.return_type.as_ref().map(|x| type_factory.link_type(&x, true)).unwrap_or_else(|| Ok(TypeProto::void()))?;

    if let [OperatorArgument::Keyword(name)] = &function.parts.iter().map(|x| x.as_ref()).collect_vec()[..] {
        // Constant

        let fun = Rc::new(FunctionPointer {
            target: FunctionHead::new(
                Rc::new(FunctionInterface {
                    parameters: vec![],
                    return_type,
                    requirements: requirements.iter().chain(&type_factory.requirements).map(Rc::clone).collect(),
                    generics: type_factory.generics,
                }),
                FunctionType::Static
            ),
            name: name.clone(),
            form: FunctionForm::GlobalImplicit,
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
            parameters.push(Parameter {
                external_key: key.clone(),
                internal_name: internal_name.to_string(),
                type_: type_factory.link_type(type_expression, true)?,
            });
        }

        return Ok(Rc::new(FunctionPointer {
            target: FunctionHead::new(
                Rc::new(FunctionInterface {
                    parameters,
                    return_type,
                    requirements: requirements.iter().chain(&type_factory.requirements).map(Rc::clone).collect(),
                    generics: type_factory.generics,
                }),
                FunctionType::Static
            ),
            name: pattern.alias.clone(),
            form: FunctionForm::GlobalFunction,
        }))
    }

    return Err(RuntimeError::new(format!("Unknown pattern in function definition: {}", function.parts.iter().map(|x| x.to_string()).join(" "))));
}

pub fn match_patterns<'a>(pattern_parts: &'a Vec<Box<PatternPart>>, function_parts: &'a Vec<Box<OperatorArgument>>) -> Option<Vec<(&'a ParameterKey, &'a str, &'a Expression)>> {
    if pattern_parts.len() != function_parts.len() {
        return None;
    }

    let mut parameters: Vec<(&ParameterKey, &str, &Expression)> = vec![];

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