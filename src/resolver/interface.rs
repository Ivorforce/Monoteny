use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use itertools::Itertools;
use try_map::FallibleMapExt;

use crate::error::{RResult, RuntimeError, TryCollectMany};
use crate::interpreter::runtime::Runtime;
use crate::resolver::scopes;
use crate::resolver::type_factory::TypeFactory;
use crate::parser::ast;
use crate::program::function_object::{FunctionCallExplicity, FunctionRepresentation, FunctionTargetType};
use crate::program::functions::{FunctionHead, FunctionInterface, Parameter};
use crate::program::module::{Module, module_name};
use crate::program::traits::{Trait, TraitBinding};
use crate::program::types::TypeProto;
use crate::resolver::grammar::Struct;
use crate::util::position::Positioned;

pub fn resolve_function_interface(interface: &ast::FunctionInterface, scope: &scopes::Scope, module: Option<&mut Module>, runtime: &Runtime, requirements: &HashSet<Rc<TraitBinding>>, generics: &HashMap<String, Rc<Trait>>) -> RResult<(Rc<FunctionHead>, FunctionRepresentation)> {
    let mut type_factory = TypeFactory::new(scope, runtime);

    match interface.expression.iter().map(|a| a.as_ref()).collect_vec()[..] {
        [
            // Macro
            Positioned { position, value: ast::Term::MacroIdentifier(m)}
        ] => {
            if !requirements.is_empty() || !generics.is_empty() {
                panic!();
            }

            resolve_macro_function_interface(module, runtime, m)
        }
        [
            // Constant-like
            Positioned { position: p1, value: ast::Term::Identifier(i)},
        ] => {
            _resolve_function_interface(FunctionRepresentation {
                name: i.clone(),
                target_type: FunctionTargetType::Global,
                call_explicity: FunctionCallExplicity::Implicit,
            }, [].into_iter(), &interface.return_type, type_factory, requirements, generics)
        }
        [
            // Function-like
            Positioned { position: p1, value: ast::Term::Identifier(i)},
            Positioned { position: p2, value: ast::Term::Struct(struct_)}
        ] => {
            _resolve_function_interface(FunctionRepresentation {
                name: i.clone(),
                target_type: FunctionTargetType::Global,
                call_explicity: FunctionCallExplicity::Explicit,
            }, struct_.arguments.iter().map(|a| &a.value), &interface.return_type, type_factory, requirements, generics)
        }
        [
            // Member-constant like
            Positioned { position: p1, value: ast::Term::Struct(target) },
            Positioned { position: p2, value: ast::Term::Dot },
            Positioned { position: p3, value: ast::Term::Identifier(member) },
        ] => {
            let target = get_as_target_parameter(target)?;
            _resolve_function_interface(FunctionRepresentation {
                name: member.clone(),
                target_type: FunctionTargetType::Member,
                call_explicity: FunctionCallExplicity::Implicit,
            }, Some(target).into_iter(), &interface.return_type, type_factory, requirements, generics)
        }
        [
            // Member-function like
            Positioned { position: p1, value: ast::Term::Struct(target) },
            Positioned { position: p2, value: ast::Term::Dot },
            Positioned { position: p3, value: ast::Term::Identifier(member) },
            Positioned { position: p4, value: ast::Term::Struct(call_struct)}
        ] => {
            let target = get_as_target_parameter(&target)?;
            _resolve_function_interface(FunctionRepresentation {
                name: member.clone(),
                target_type: FunctionTargetType::Member,
                call_explicity: FunctionCallExplicity::Explicit,
            }, Some(target).into_iter().chain(call_struct.arguments.iter().map(|a| &a.value)), &interface.return_type, type_factory, requirements, generics)
        }
        _ => Err(RuntimeError::error("Cannot have non-function definition.").to_array()),
    }
}

fn resolve_macro_function_interface(module: Option<&mut Module>, runtime: &Runtime, m: &String) -> RResult<(Rc<FunctionHead>, FunctionRepresentation)> {
    match m.as_str() {
        "main" => {
            let proto_function = runtime.source.module_by_name[&module_name("core.run")].explicit_functions(&runtime.source).into_iter()
                .filter(|function| runtime.source.fn_representations[*function].name == "main")
                .exactly_one().unwrap();

            let fun = FunctionHead::new_static(Rc::clone(&proto_function.interface));
            let representation = FunctionRepresentation::new("main", FunctionTargetType::Global, FunctionCallExplicity::Explicit);

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
            let representation = FunctionRepresentation::new("transpile", FunctionTargetType::Global, FunctionCallExplicity::Explicit);

            if let Some(module) = module {
                module.transpile_functions.push(Rc::clone(&fun));
            }
            Ok((fun, representation))
        },
        _ => Err(
            RuntimeError::error(format!("Function macro could not be resolved: {}", m).as_str()).to_array()
        ),
    }
}

pub fn _resolve_function_interface<'a>(representation: FunctionRepresentation, parameters: impl Iterator<Item=&'a ast::StructArgument>, return_type: &Option<ast::Expression>, mut type_factory: TypeFactory, requirements: &HashSet<Rc<TraitBinding>>, generics: &HashMap<String, Rc<Trait>>) -> RResult<(Rc<FunctionHead>, FunctionRepresentation)> {
    let return_type = return_type.as_ref()
        .try_map(|x| type_factory.resolve_type(&x, true))?
        .unwrap_or(TypeProto::void());

    let parameters = parameters
        .map(|p| resolve_function_parameter(p, &mut type_factory))
        .try_collect_many()?;

    let mut generics = generics.clone();
    generics.extend(type_factory.generics);

    let requirements = requirements.iter()
        .chain(&type_factory.requirements)
        .map(Rc::clone)
        .collect();

    Ok((
        FunctionHead::new_static(
            Rc::new(FunctionInterface {
                parameters,
                return_type,
                requirements,
                generics,
            }),
        ),
        representation
    ))
}

pub fn resolve_function_parameter(parameter: &ast::StructArgument, type_factory: &mut TypeFactory) -> RResult<Parameter> {
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

    Ok(Parameter {
        external_key: parameter.key.clone(),
        internal_name: internal_name.clone(),
        type_: type_factory.resolve_type(type_declaration, true)?,
    })
}

pub fn get_as_target_parameter(term: &ast::Struct) -> RResult<&ast::StructArgument> {
    let [target] = &term.arguments[..] else {
        return Err(
            RuntimeError::error("Target of member function must be one-element struct.").to_array()
        )
    };

    Ok(&target.value)
}
