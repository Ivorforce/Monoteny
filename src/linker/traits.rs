use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use crate::error::{RResult, RuntimeError};
use crate::linker::scopes;
use crate::linker::interface::link_function_pointer;
use crate::linker::r#type::TypeFactory;
use crate::linker::scopes::Environment;
use crate::parser::ast;
use crate::program::allocation::Mutability;
use crate::program::functions::{FunctionForm, FunctionHead, FunctionInterface, FunctionPointer, FunctionType, Parameter, ParameterKey};
use crate::program::global::BuiltinFunctionHint;
use crate::program::module::Module;
use crate::program::traits::{Trait, TraitBinding, TraitConformance, TraitConformanceRule, VariableHint};
use crate::program::types::{TypeProto, TypeUnit};

pub struct TraitLinker<'a> {
    pub trait_: &'a mut Trait,
}

impl <'a> TraitLinker<'a> {
    pub fn link_statement(&mut self, statement: &'a ast::Statement, requirements: &HashSet<Rc<TraitBinding>>, scope: &scopes::Scope) -> RResult<()> {
        match statement {
            ast::Statement::FunctionDeclaration(syntax) => {
                let fun = link_function_pointer(&syntax, &scope, requirements)?;
                if !syntax.body.is_none() {
                    return Err(RuntimeError::new(format!("Abstract function {} cannot have a body.", fun.name)));
                };

                self.trait_.insert_function(fun);
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

                let mut type_factory = TypeFactory::new(scope);

                let variable_type = type_factory.link_type(type_declaration)?;
                let trait_type = scope.resolve(Environment::Global, "Self")?.as_metatype()?;

                if TypeProto::contains_generics([&variable_type].into_iter()) {
                    return Err(RuntimeError::new(format!("Variables cannot be generic: {}", identifier)));
                }

                let getter = make_getter(trait_type.clone(), identifier, variable_type.clone());
                self.trait_.insert_function(Rc::clone(&getter));

                let setter = match mutability {
                    Mutability::Immutable => None,
                    Mutability::Mutable => {
                        let setter = make_setter(trait_type.clone(), identifier, variable_type.clone());
                        self.trait_.insert_function(Rc::clone(&setter));
                        Some(Rc::clone(&setter.target))
                    }
                };

                self.trait_.variable_hints.push(VariableHint {
                    name: identifier.clone(),
                    setter,
                    getter: Some(Rc::clone(&getter.target)),
                    type_: variable_type,
                });
            }
            _ => {
                return Err(RuntimeError::new(format!("Statement {} not valid in a trait context.", statement)));
            }
        }

        Ok(())
    }
}

fn make_setter(struct_type: Box<TypeProto>, identifier: &str, variable_type: Box<TypeProto>) -> Rc<FunctionPointer> {
    Rc::new(FunctionPointer {
        target: FunctionHead::new(
            Rc::new(FunctionInterface {
                parameters: vec![Parameter {
                    external_key: ParameterKey::Positional,
                    internal_name: "self".to_string(),
                    type_: struct_type,
                }, Parameter {
                    external_key: ParameterKey::Positional,
                    internal_name: identifier.to_string(),
                    type_: variable_type,
                }],
                return_type: TypeProto::void(),
                requirements: Default::default(),
            }),
            FunctionType::Static
        ),
        name: identifier.to_string(),
        form: FunctionForm::MemberField,
    })
}

fn make_getter(struct_type: Box<TypeProto>, identifier: &str, variable_type: Box<TypeProto>) -> Rc<FunctionPointer> {
    Rc::new(FunctionPointer {
        target: FunctionHead::new(
            Rc::new(FunctionInterface {
                parameters: vec![
                    Parameter {
                    external_key: ParameterKey::Positional,
                    internal_name: "self".to_string(),
                    type_: struct_type,
                }],
                return_type: variable_type.clone(),
                requirements: Default::default(),
            }),
            FunctionType::Static
        ),
        name: identifier.to_string(),
        form: FunctionForm::MemberField,
    })
}

pub fn try_make_struct(trait_: &Rc<Trait>, module: &mut Module, scope: &mut scopes::Scope) -> RResult<()> {
    let mut unaccounted_for_abstract_functions: HashSet<_> = trait_.abstract_functions.keys().collect();
    trait_.variable_hints.iter().for_each(|hint| {
        [&hint.getter, &hint.setter].into_iter().flatten().map(|g| unaccounted_for_abstract_functions.remove(g)).collect_vec();
    });

    if !unaccounted_for_abstract_functions.is_empty() {
        return Ok(())
    }

    // Can be instantiated as a struct!

    let struct_type = TypeProto::unit(TypeUnit::Struct(Rc::clone(&trait_)));
    let mut function_mapping = HashMap::new();
    let mut parameters = vec![
        Parameter {
            external_key: ParameterKey::Positional,
            internal_name: "type".to_string(),
            type_: TypeProto::meta(struct_type.clone()),
        }
    ];

    for hint in trait_.variable_hints.iter() {
        // TODO Once generic types are supported, the variable type should be mapped to actual types
        if let Some(abstract_getter) = &hint.getter {
            let struct_getter = make_getter(struct_type.clone(), hint.name.as_str(), hint.type_.clone());
            module.fn_builtin_hints.insert(Rc::clone(&struct_getter.target), BuiltinFunctionHint::Getter);
            module.add_function(&struct_getter);
            scope.overload_function(&struct_getter, &module.fn_references[&struct_getter.target])?;
            function_mapping.insert(Rc::clone(abstract_getter), Rc::clone(&struct_getter.target));
        }
        if let Some(abstract_setter) = &hint.setter {
            let struct_setter = make_setter(struct_type.clone(), hint.name.as_str(), hint.type_.clone());
            module.fn_builtin_hints.insert(Rc::clone(&struct_setter.target), BuiltinFunctionHint::Getter);
            module.add_function(&struct_setter);
            scope.overload_function(&struct_setter, &module.fn_references[&struct_setter.target])?;
            function_mapping.insert(Rc::clone(abstract_setter), Rc::clone(&struct_setter.target));
        }

        parameters.push(Parameter {
            external_key: ParameterKey::Name(hint.name.clone()),
            internal_name: hint.name.clone(),
            type_: hint.type_.clone(),
        });
    }

    let conformance = TraitConformance::new(
        trait_.create_generic_binding(vec![("Self", struct_type.clone())]),
        function_mapping,
    );
    module.trait_conformance.add_conformance_rule(TraitConformanceRule::direct(Rc::clone(&conformance)));
    scope.traits.add_conformance_rule(TraitConformanceRule::direct(conformance));

    let new_function = Rc::new(FunctionPointer {
        target: FunctionHead::new(
            Rc::new(FunctionInterface {
                parameters,
                return_type: struct_type,
                requirements: Default::default(),
            }),
            FunctionType::Static
        ),
        name: "call_as_function".to_string(),
        form: FunctionForm::MemberFunction,
    });
    module.add_function(&new_function);
    module.fn_builtin_hints.insert(Rc::clone(&new_function.target), BuiltinFunctionHint::Constructor);
    scope.overload_function(&new_function, &module.fn_references[&new_function.target])?;

    Ok(())
}
