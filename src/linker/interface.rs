use std::collections::HashSet;
use std::rc::Rc;
use guard::guard;
use itertools::{Itertools, zip_eq};
use crate::error::{RResult, RuntimeError};
use crate::interpreter::Runtime;
use crate::linker::type_factory::TypeFactory;
use crate::linker::scopes;
use crate::parser::ast;
use crate::program::function_object::{FunctionForm, FunctionRepresentation};
pub use crate::program::functions::{FunctionHead, FunctionInterface, Parameter, ParameterKey};
use crate::program::module::{Module, module_name};
use crate::program::traits::TraitBinding;
use crate::program::types::{PatternPart, TypeProto};


pub fn link_function_interface(interface: &ast::FunctionInterface, scope: &scopes::Scope, module: Option<&mut Module>, runtime: &Runtime, requirements: &HashSet<Rc<TraitBinding>>) -> RResult<(Rc<FunctionHead>, FunctionRepresentation)> {
    match interface {
        ast::FunctionInterface::Macro(m) => {
            match m.as_str() {
                "main" => {
                    let proto_function = runtime.source.module_by_name[&module_name("core.run")].explicit_functions(&runtime.source).into_iter()
                        .filter(|function| runtime.source.fn_representations[*function].name == "main")
                        .exactly_one().unwrap();

                    let fun = FunctionHead::new_static(Rc::clone(&proto_function.interface));
                    let representation = FunctionRepresentation::new("main", FunctionForm::GlobalFunction);

                    if let Some(module) = module {
                        module.main_functions.push(Rc::clone(&fun));
                    }
                    Ok((fun, representation))
                },
                "transpile" => {
                    let proto_function = runtime.source.module_by_name[&module_name("core.transpilation")].explicit_functions(&runtime.source).into_iter()
                        .filter(|function| runtime.source.fn_representations[*function].name == "transpile")
                        .exactly_one().unwrap();

                    let fun = FunctionHead::new_static(Rc::clone(&proto_function.interface));
                    let representation = FunctionRepresentation::new("transpile", FunctionForm::GlobalFunction);

                    if let Some(module) = module {
                        module.transpile_functions.push(Rc::clone(&fun));
                    }
                    Ok((fun, representation))
                },
                _ => Err(RuntimeError::new(format!("Function macro could not be resolved: {}", m))),
            }
        }
        ast::FunctionInterface::Explicit { identifier, target_type, parameters, return_type } => {
            let mut type_factory = TypeFactory::new(scope, runtime);
            let return_type = return_type.as_ref().map(|x| type_factory.link_type(&x, true)).unwrap_or_else(|| Ok(TypeProto::void()))?;

            let mut fn_parameters: Vec<Parameter> = vec![];

            if let Some(parameter) = &target_type {
                fn_parameters.push(Parameter {
                    external_key: ParameterKey::Positional,
                    internal_name: String::from("self"),
                    type_: type_factory.link_type(parameter, true)?,
                });
            }

            for parameter in parameters.iter() {
                fn_parameters.push(Parameter {
                    external_key: parameter.key.clone(),
                    internal_name: parameter.internal_name.clone(),
                    type_: type_factory.link_type(&parameter.param_type, true)?,
                });
            }

            Ok((
                FunctionHead::new_static(
                    Rc::new(FunctionInterface {
                        parameters: fn_parameters,
                        return_type,
                        requirements: requirements.iter().chain(&type_factory.requirements).map(Rc::clone).collect(),
                        generics: type_factory.generics,
                    }),
                ),
                FunctionRepresentation::new(
                    identifier,
                    if target_type.is_none() { FunctionForm::GlobalFunction } else { FunctionForm::MemberFunction }
                )
            ))
        }
    }
}

pub fn link_operator_interface(function: &ast::OperatorFunction, scope: &scopes::Scope, runtime: &Runtime, requirements: &HashSet<Rc<TraitBinding>>) -> RResult<(Rc<FunctionHead>, FunctionRepresentation)> {
    let mut type_factory = TypeFactory::new(scope, runtime);

    let return_type = function.return_type.as_ref().map(|x| type_factory.link_type(&x, true)).unwrap_or_else(|| Ok(TypeProto::void()))?;

    if let [ast::OperatorArgument::Keyword(name)] = &function.parts.iter().map(|x| x.as_ref()).collect_vec()[..] {
        // Constant

        return Ok((
            FunctionHead::new_static(
                Rc::new(FunctionInterface {
                    parameters: vec![],
                    return_type,
                    requirements: requirements.iter().chain(&type_factory.requirements).map(Rc::clone).collect(),
                    generics: type_factory.generics,
                }),
            ),
            FunctionRepresentation::new(
                name,
                FunctionForm::GlobalImplicit
            )
        ))
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

        return Ok((
            FunctionHead::new_static(
                Rc::new(FunctionInterface {
                    parameters,
                    return_type,
                    requirements: requirements.iter().chain(&type_factory.requirements).map(Rc::clone).collect(),
                    generics: type_factory.generics,
                }),
            ),
            FunctionRepresentation::new(
                &pattern.alias,
                FunctionForm::GlobalFunction,
            )
        ))
    }

    return Err(RuntimeError::new(format!("Unknown pattern in function definition: {}", function.parts.iter().map(|x| x.to_string()).join(" "))));
}

pub fn match_patterns<'a>(pattern_parts: &'a Vec<Box<PatternPart>>, function_parts: &'a Vec<Box<ast::OperatorArgument>>) -> Option<Vec<(&'a ParameterKey, &'a str, &'a ast::Expression)>> {
    if pattern_parts.len() != function_parts.len() {
        return None;
    }

    let mut parameters: Vec<(&ParameterKey, &str, &ast::Expression)> = vec![];

    for (pattern_part, function_part) in zip_eq(pattern_parts.iter(), function_parts.iter()) {
        match (pattern_part.as_ref(), function_part.as_ref()) {
            (PatternPart::Keyword(k1), ast::OperatorArgument::Keyword(k2)) => {
                if k1 != k2 {
                    return None;
                }
            }
            (PatternPart::Parameter { key, internal_name }, ast::OperatorArgument::Parameter(expression)) => {
                parameters.push((key, internal_name, expression));
            }
            _ => return None,
        }
    }

    Some(parameters)
}