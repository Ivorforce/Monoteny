pub mod computation_tree;
pub mod scopes;
pub mod imperative;

use std::collections::HashMap;
use std::iter::zip;
use std::rc::Rc;
use guard::guard;
use uuid::Uuid;

use crate::parser;
use crate::parser::abstract_syntax;
use crate::linker::computation_tree::*;
use crate::linker::imperative::ImperativeResolver;
use crate::program::primitives;
use crate::program::builtins::*;
use crate::program::types::*;


pub fn link_program(syntax: abstract_syntax::Program, parser_scope: &parser::scopes::Level, scope: &scopes::Hierarchy, builtins: &TenLangBuiltins) -> Program {
    let mut functions_with_bodies: Vec<(Rc<FunctionInterface>, &Vec<Box<abstract_syntax::Statement>>)> = Vec::new();
    let mut global_variables = scopes::Level::new();

    // Resolve things in global scope
    for statement in &syntax.global_statements {
        match statement.as_ref() {
            abstract_syntax::GlobalStatement::FunctionDeclaration(function) => {
                let interface = link_function_interface(&function, &scope);

                functions_with_bodies.push((Rc::clone(&interface), &function.body));

                let environment = match interface.form {
                    FunctionForm::Member => scopes::Environment::Member,
                    _ => scopes::Environment::Global,
                };

                // Create a variable for the function
                global_variables.add_function(environment, Rc::new(Variable {
                    id: Uuid::new_v4(),
                    name: interface.name.clone(),
                    type_declaration: Box::new(Type::Function(Rc::clone(&interface))),
                    mutability: Mutability::Immutable,
                }));

                // if interface.is_member_function {
                // TODO Create an additional variable as Metatype.function...?
                // }
            }
            abstract_syntax::GlobalStatement::Operator(operator) => {
                let interface = link_operator_interface(&operator, parser_scope, &scope);

                functions_with_bodies.push((Rc::clone(&interface), &operator.body));

                // Create a variable for the function
                global_variables.add_function(scopes::Environment::Global, Rc::new(Variable {
                    id: Uuid::new_v4(),
                    name: interface.name.clone(),
                    type_declaration: Box::new(Type::Function(Rc::clone(&interface))),
                    mutability: Mutability::Immutable,
                }));
            }
            _ => {}
        }
    }

    let global_variable_scope = scope.subscope(&global_variables);

    // Resolve function bodies
    let functions: Vec<Rc<Function>> = functions_with_bodies.into_iter().map(
        |(interface, statements)| {
            let resolver = ImperativeResolver {
                interface: &interface,
                builtins
            };
            resolver.link_function_body(statements, &global_variable_scope)
        }
    ).collect();

    return Program {
        functions,
    }
}

pub fn link_function_interface(function: &abstract_syntax::Function, scope: &scopes::Hierarchy) -> Rc<FunctionInterface> {
    let return_type = function.return_type.as_ref().map(|x| link_type(&x, scope));

    let mut parameters: Vec<Box<NamedParameter>> = Vec::new();

    if let Some(target) = &function.target {
        parameters.push(Box::new(NamedParameter {
            external_key: ParameterKey::Int(0),
            variable: link_contextual_parameter_as_variable(target, scope)
        }));
    }

    for parameter in function.parameters.iter() {
        let variable = Rc::new(Variable {
            id: Uuid::new_v4(),
            name: parameter.internal_name.clone(),
            type_declaration: link_type(parameter.param_type.as_ref(), scope),
            mutability: Mutability::Immutable,
        });

        parameters.push(Box::new(NamedParameter {
            external_key: link_parameter_key(&parameter.key, parameters.len()),
            variable
        }));
    }

    return Rc::new(FunctionInterface {
        id: Uuid::new_v4(),
        name: function.identifier.clone(),
        alphanumeric_name: function.identifier.clone(),

        form: if function.target.is_none() { FunctionForm::Global } else { FunctionForm::Member },
        parameters,
        // This is correct so far because syntax does not allow generics use yet.
        generics: vec![],

        return_type
    });
}

pub fn link_operator_interface(function: &abstract_syntax::Operator, parser_scope: &parser::scopes::Level, scope: &scopes::Hierarchy) -> Rc<FunctionInterface> {
    let return_type = function.return_type.as_ref().map(|x| link_type(&x, scope));

    let parameters: Vec<Box<NamedParameter>> = function.lhs.iter().chain([&function.rhs]).enumerate()
        .map(|(idx, x)| {
            Box::new(NamedParameter {
                external_key: ParameterKey::Int(idx as i32),
                variable: link_contextual_parameter_as_variable(x, scope)
            })
        })
        .collect();

    let is_binary = function.lhs.is_some();
    let pattern = parser_scope.resolve_operator_pattern(&function.operator, is_binary);

    return Rc::new(FunctionInterface {
        id: Uuid::new_v4(),
        name: function.operator.clone(),
        alphanumeric_name: pattern.alias.clone(),

        form: FunctionForm::Operator,
        parameters,
        // This is correct so far because syntax does not allow generics use yet.
        generics: vec![],

        return_type
    });
}

pub fn link_contextual_parameter_as_variable(parameter: &abstract_syntax::ContextualParameter, scope: &scopes::Hierarchy) -> Rc<Variable> {
    Rc::new(Variable {
        id: Uuid::new_v4(),
        name: String::from(&parameter.internal_name),
        type_declaration: link_type(&parameter.param_type, scope),
        mutability: Mutability::Immutable,
    })
}

pub fn link_parameter_key(key: &ParameterKey, index: usize) -> ParameterKey {
    match key {
        ParameterKey::Int(n) => ParameterKey::Int(*n),
        ParameterKey::Name(n) => {
            match n.as_str() {
                // When _ a: SomeType is declared, it is keyed by its index.
                // TODO it should be keyed by the previous index +1 instead
                "_" => ParameterKey::Int(index as i32),
                _ => ParameterKey::Name(n.clone())
            }
        },
    }
}

pub fn link_type(syntax: &abstract_syntax::TypeDeclaration, scope: &scopes::Hierarchy) -> Box<Type> {
    match syntax {
        abstract_syntax::TypeDeclaration::Identifier(id) => {
            scope.resolve_metatype(scopes::Environment::Global, id).clone()
        },
        abstract_syntax::TypeDeclaration::Monad { unit, shape } => {
            Box::new(Type::Monad(link_type(&unit, scope)))
        }
    }
}
