use std::collections::HashSet;
use std::rc::Rc;

use itertools::Itertools;
use try_map::FallibleMapExt;

use crate::ast;
use crate::error::{RResult, RuntimeError};
use crate::interpreter::runtime::Runtime;
use crate::parser::expressions;
use crate::program::functions::{FunctionCallExplicity, FunctionHead, FunctionInterface, FunctionRepresentation, FunctionTargetType, Parameter};
use crate::program::module::{Module, module_name};
use crate::program::traits::{Trait, TraitBinding};
use crate::program::types::TypeProto;
use crate::resolver::scopes;
use crate::resolver::type_factory::TypeFactory;
use crate::util::position::Positioned;

pub fn resolve_function_interface(interface: &ast::FunctionInterface, scope: &scopes::Scope, module: Option<&mut Module>, runtime: &mut Runtime, requirements: &HashSet<Rc<TraitBinding>>, generics: &HashSet<Rc<Trait>>) -> RResult<Rc<FunctionHead>> {
    let mut type_factory = TypeFactory::new(scope);

    let parsed = expressions::parse(&interface.expression, &scope.grammar)?;

    match &parsed.value {
        expressions::Value::MacroIdentifier(macro_name) => {
            // Macro
            if !requirements.is_empty() || !generics.is_empty() {
                panic!();
            }

            resolve_macro_function_interface(module, runtime, macro_name)
        }
        expressions::Value::Identifier(identifier) => {
            // Constant like
            _resolve_function_interface(FunctionRepresentation {
                name: identifier.to_string(),
                target_type: FunctionTargetType::Global,
                call_explicity: FunctionCallExplicity::Implicit,
            }, [].into_iter(), &interface.return_type, type_factory, requirements, generics, runtime)
        }
        expressions::Value::MemberAccess(target, member) => {
            // Member constant like
            let target = get_as_target_parameter(&target.value)?;
            _resolve_function_interface(FunctionRepresentation {
                name: member.to_string(),
                target_type: FunctionTargetType::Member,
                call_explicity: FunctionCallExplicity::Implicit,
            }, Some(target).into_iter(), &interface.return_type, type_factory, requirements, generics, runtime)
        }
        expressions::Value::FunctionCall(target, call_struct) => {
            match &target.value {
                expressions::Value::Identifier(identifier) => {
                    // Function like
                    _resolve_function_interface(FunctionRepresentation {
                        name: identifier.to_string(),
                        target_type: FunctionTargetType::Global,
                        call_explicity: FunctionCallExplicity::Explicit,
                    }, call_struct.arguments.iter().map(|a| &a.value), &interface.return_type, type_factory, requirements, generics, runtime)
                }
                expressions::Value::MemberAccess(target, member) => {
                    // Member function like
                    let target = get_as_target_parameter(&target.value)?;
                    _resolve_function_interface(FunctionRepresentation {
                        name: member.to_string(),
                        target_type: FunctionTargetType::Member,
                        call_explicity: FunctionCallExplicity::Explicit,
                    }, Some(target).into_iter().chain(call_struct.arguments.iter().map(|a| &a.value)), &interface.return_type, type_factory, requirements, generics, runtime)
                }
                _ => return Err(RuntimeError::error("Invalid function definition.").to_array()),
            }
        }
        _ => return Err(RuntimeError::error("Invalid function definition.").to_array()),
    }
}

fn resolve_macro_function_interface(module: Option<&mut Module>, runtime: &Runtime, macro_name: &str) -> RResult<Rc<FunctionHead>> {
    match macro_name {
        "main" => {
            let proto_function = runtime.source.module_by_name[&module_name("core.run")].explicit_functions(&runtime.source).into_iter()
                .filter(|function| function.declared_representation.name == "main")
                .exactly_one().unwrap();

            let function = FunctionHead::new_static(
                proto_function.declared_internal_parameter_names.clone(),
                proto_function.declared_representation.clone(),
                Rc::clone(&proto_function.interface),
            );

            if let Some(module) = module {
                module.main_functions.push(Rc::clone(&function));
            }
            Ok(function)
        },
        "transpile" => {
            let proto_function = runtime.source.module_by_name[&module_name("core.transpilation")].explicit_functions(&runtime.source).into_iter()
                .filter(|function| function.declared_representation.name == "transpile")
                .exactly_one().unwrap();

            let function = FunctionHead::new_static(
                proto_function.declared_internal_parameter_names.clone(),
                proto_function.declared_representation.clone(),
                Rc::clone(&proto_function.interface)
            );

            if let Some(module) = module {
                module.transpile_functions.push(Rc::clone(&function));
            }
            Ok(function)
        },
        _ => Err(
            RuntimeError::error(format!("Function macro could not be resolved: {}", macro_name).as_str()).to_array()
        ),
    }
}

pub fn _resolve_function_interface<'a>(representation: FunctionRepresentation, parameters: impl Iterator<Item=&'a ast::StructArgument>, return_type: &Option<ast::Expression>, mut type_factory: TypeFactory, requirements: &HashSet<Rc<TraitBinding>>, generics: &HashSet<Rc<Trait>>, runtime: &mut Runtime) -> RResult<Rc<FunctionHead>> {
    let return_type = return_type.as_ref()
        .try_map(|x| type_factory.resolve_type(&x, true, runtime))?
        .unwrap_or(TypeProto::void());

    let (parameters, internal_parameter_names) = parameters
        .map(|p| resolve_function_parameter(p, &mut type_factory, runtime))
        .process_results(|i| i.unzip())?;

    let mut generics = generics.clone();
    generics.extend(type_factory.generics.values().cloned().collect_vec());

    let requirements = requirements.iter()
        .chain(&type_factory.requirements)
        .map(Rc::clone)
        .collect();

    let interface = FunctionInterface {
        parameters,
        return_type,
        requirements,
        generics,
    };

    Ok(FunctionHead::new_static(
        internal_parameter_names,
        representation,
        Rc::new(interface),
    ))
}

pub fn resolve_function_parameter(parameter: &ast::StructArgument, type_factory: &mut TypeFactory, runtime: &mut Runtime) -> RResult<(Parameter, String)> {
    let Some(type_declaration) = &parameter.type_declaration else {
        return Err(
            RuntimeError::error("Parameters must have a type.").to_array()
        );
    };

    let [
        Positioned { position, value: ast::Term::Identifier(internal_name) }
    ] = parameter.value.iter().map(|a| a.as_ref()).collect_vec()[..] else {
        return Err(
            RuntimeError::error("Cannot have non-identifier internal name.").to_array()
        )
    };

    Ok((Parameter {
        external_key: parameter.key.clone(),
        type_: type_factory.resolve_type(type_declaration, true, runtime)?,
    }, internal_name.clone()))
}

pub fn get_as_target_parameter<'a>(term: &'a expressions::Value<Rc<FunctionHead>>) -> RResult<&'a ast::StructArgument> {
    let expressions::Value::StructLiteral(struct_) = term else {
        return Err(RuntimeError::error("Target of member function must be one-element struct.").to_array())
    };

    let [target] = &struct_.arguments[..] else {
        return Err(RuntimeError::error("Target of member function must be one-element struct.").to_array())
    };

    Ok(&target.value)
}
