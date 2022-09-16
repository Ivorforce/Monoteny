use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use guard::guard;
use itertools::zip_eq;
use uuid::Uuid;

use crate::parser;
use crate::parser::abstract_syntax;
use crate::program::computation_tree::*;
use crate::linker::imperative::ImperativeLinker;
use crate::linker::scopes;
use crate::parser::abstract_syntax::Function;
use crate::program::traits::{TraitConformanceDeclaration, TraitConformanceRequirement};
use crate::program::primitives;
use crate::program::builtins::*;
use crate::program::functions::{FunctionForm, FunctionPointer, HumanFunctionInterface, MachineFunctionInterface};
use crate::program::generics::GenericMapping;
use crate::program::types::*;


struct GlobalLinker<'a> {
    functions: Vec<FunctionWithoutBody<'a>>,
    global_variables: scopes::Level,
    parser_scope: &'a parser::scopes::Level,
    builtins: &'a TenLangBuiltins,
}

struct FunctionWithoutBody<'a> {
    pointer: Rc<FunctionPointer>,
    body: &'a Vec<Box<abstract_syntax::Statement>>,
    injected_pointers: HashSet<Rc<FunctionPointer>>
}

pub fn link_file(syntax: abstract_syntax::Program, parser_scope: &parser::scopes::Level, scope: &scopes::Hierarchy, builtins: &TenLangBuiltins) -> Program {
    let mut global_linker = GlobalLinker {
        functions: Vec::new(),
        global_variables: scopes::Level::new(),
        parser_scope,
        builtins
    };

    // Alias global names
    // TODO

    // Resolve global types / interfaces
    for statement in &syntax.global_statements {
        global_linker.link_global_statement(statement.as_ref(), scope, &HashSet::new());
    }

    let global_variable_scope = scope.subscope(&global_linker.global_variables);

    // Resolve function bodies
    let functions: HashSet<Rc<FunctionImplementation>> = global_linker.functions.iter().map(
        |fun| {
            let mut variable_names = HashMap::new();
            for (name, (_, variable)) in zip_eq(fun.pointer.human_interface.parameter_names_internal.iter(), fun.pointer.human_interface.parameter_names.iter()) {
                variable_names.insert(Rc::clone(variable), name.clone());
            }

            let mut resolver = Box::new(ImperativeLinker {
                function: Rc::clone(&fun.pointer),
                builtins,
                generics: GenericMapping::new(),
                variable_names,
                injected_pointers: fun.injected_pointers.clone(),
                used_functions: HashSet::new(),
            });

            let mut injection_level = scopes::Level::new();
            for fun in fun.injected_pointers.iter() {
                injection_level.add_function(Rc::clone(fun));
            }

            let function_scope = global_variable_scope.subscope(&injection_level);

            resolver.link_function_body(fun.body, &function_scope)
        }
    ).collect();

    return Program {
        functions,
    }
}

impl <'a> GlobalLinker<'a> {
    pub fn link_global_statement(&mut self, statement: &'a abstract_syntax::GlobalStatement, scope: &scopes::Hierarchy, requirements: &HashSet<Rc<TraitConformanceRequirement>>) {
        match statement {
            abstract_syntax::GlobalStatement::Scope(syntax) => {
                let mut level_with_generics = scopes::Level::new();

                for generic_name in syntax.generics.iter().flat_map(|x| x.iter()) {
                    level_with_generics.insert_singleton(scopes::Environment::Global, Variable::make_immutable(Type::meta(Type::make_any())), generic_name)
                }

                let subscope = scope.subscope(&level_with_generics);

                with_requirements(&subscope, syntax.requirements.iter().flat_map(|x| x.iter()).map(|x| x.as_ref()), requirements, |scope, requirements| {
                    for statement in &syntax.statements {
                        self.link_global_statement(statement.as_ref(), scope, &requirements);
                    }
                });
            }
            abstract_syntax::GlobalStatement::FunctionDeclaration(syntax) => {
                with_anonymous_generics(&syntax.gather_type_names(), scope, requirements, |scope, requirements| {
                    let fun = link_function_pointer(&syntax, scope, requirements.clone());

                    self.functions.push(FunctionWithoutBody {
                        pointer: Rc::clone(&fun),
                        body: &syntax.body,
                        injected_pointers: requirements.iter()
                            .flat_map(|x| x.functions_pointers.iter().map(|x| Rc::clone(&x.1)))
                            .collect()
                    });

                    // Create a variable for the function
                    self.global_variables.add_function(fun);

                    // if interface.is_member_function {
                    // TODO Create an additional variable as Metatype.function...?
                    // }
                });
            }
            abstract_syntax::GlobalStatement::Operator(syntax) => {
                with_anonymous_generics(&syntax.gather_type_names(), scope, requirements, |scope, requirements| {
                    let fun = link_operator_pointer(&syntax, self.parser_scope, scope, requirements.clone());

                    self.functions.push(FunctionWithoutBody {
                        pointer: Rc::clone(&fun),
                        body: &syntax.body,
                        injected_pointers: requirements.iter()
                            .flat_map(|x| x.functions_pointers.iter().map(|x| Rc::clone(&x.1)))
                            .collect()
                    });

                    // Create a variable for the function
                    self.global_variables.add_function(fun);
                });
            }
            _ => {}
        }
    }
}

pub fn with_anonymous_generics<'a, F>(type_names: &'a Vec<&'a String>, scope: &scopes::Hierarchy, requirements: &HashSet<Rc<TraitConformanceRequirement>>, fun: F) where F: FnOnce(&scopes::Hierarchy, &HashSet<Rc<TraitConformanceRequirement>>) {
    let mut level = Box::new(scopes::Level::new());
    let mut requirements_syntax = Vec::new();

    let mut needs_scope = false;
    let mut needs_requirements = false;

    for type_name in type_names {
        if type_name.starts_with("$") && !level.contains(scopes::Environment::Global, type_name) {
            level.insert_singleton(scopes::Environment::Global, Variable::make_immutable(Type::meta(Type::make_any())), type_name);
            needs_scope = true;

            let trait_name = String::from(&type_name[1..]);
            if trait_name.starts_with("Any") {
                continue
            }

            requirements_syntax.push(abstract_syntax::TraitDeclaration {
                unit: trait_name,
                elements: vec![Box::new(abstract_syntax::SpecializedType {
                    unit: (*type_name).clone(),
                    elements: None
                })]
            });
            needs_requirements = true;
        }
    }

    if needs_scope {
        let subscope = scope.subscope(&level);

        if needs_requirements {
            with_requirements(&subscope, requirements_syntax.iter(), requirements, fun);
        }
        else {
            fun(&subscope, requirements);
        }
    }
    else {
        fun(&scope, requirements);
    }
}

pub fn with_requirements<'a, I, F>(scope: &scopes::Hierarchy, requirements_syntax: I, requirements: &HashSet<Rc<TraitConformanceRequirement>>, fun: F) where F: FnOnce(&scopes::Hierarchy, &HashSet<Rc<TraitConformanceRequirement>>), I: Iterator<Item=&'a abstract_syntax::TraitDeclaration> {
    let mut level_with_requirements = scopes::Level::new();

    let mut requirements = requirements.clone();
    for requirement_syntax in requirements_syntax {
        let trait_ = Rc::clone(scope.resolve_trait(scopes::Environment::Global, &requirement_syntax.unit));
        let arguments: Vec<Box<Type>> = requirement_syntax.elements.iter().map(|x| {
            link_specialized_type(x, &scope)
        }).collect();

        let mut replace_map = HashMap::new();
        for (param, arg) in zip_eq(trait_.parameters.iter(), arguments.iter()) {
            replace_map.insert(param.clone(), arg.clone());
        }

        let mut functions_pointers = HashMap::new();

        // Add requirement's implied abstract functions to scope
        for abstract_fun in trait_.abstract_functions.iter() {
            let mapped_pointer = Rc::new(FunctionPointer {
                pointer_id: Uuid::new_v4(),
                function_id: abstract_fun.function_id,
                human_interface: Rc::clone(&abstract_fun.human_interface),
                machine_interface: Rc::new(MachineFunctionInterface {
                    // TODO Mapping variables seems wrong, especially since they are hashable by ID?
                    //  Parameters should probably not point to variables directly.
                    parameters: abstract_fun.machine_interface.parameters.iter().map(|x| Rc::new(Variable {
                        id: x.id,
                        type_declaration: x.type_declaration.replacing_any(&replace_map),
                        mutability: x.mutability
                    })).collect(),
                    return_type: abstract_fun.machine_interface.return_type.as_ref().map(|x| x.replacing_any(&replace_map)),
                    injectable_pointers: abstract_fun.machine_interface.injectable_pointers.clone(),
                })
            });

            functions_pointers.insert(Rc::clone(abstract_fun), Rc::clone(&mapped_pointer));
            level_with_requirements.add_function(mapped_pointer);
        }

        // Add requirement to scope, which is used for declarations like trait conformance and functions
        requirements.insert(Rc::new(TraitConformanceRequirement {
            id: Uuid::new_v4(),
            trait_: Rc::clone(&trait_),
            arguments,
            functions_pointers
        }));
    }

    let subscope = scope.subscope(&level_with_requirements);

    fun(&subscope, &requirements);
}

pub fn link_function_pointer(function: &abstract_syntax::Function, scope: &scopes::Hierarchy, requirements: HashSet<Rc<TraitConformanceRequirement>>) -> Rc<FunctionPointer> {
    let return_type = function.return_type.as_ref().map(|x| link_type(&x, scope));

    let mut parameters: HashSet<Rc<Variable>> = HashSet::new();
    let mut parameter_names: Vec<(ParameterKey, Rc<Variable>)> = vec![];
    let mut parameter_names_internal: Vec<String> = vec![];

    if let Some(parameter) = &function.target {
        let variable = Variable::make_immutable(link_type(&parameter.param_type, scope));

        parameters.insert(Rc::clone(&variable));
        parameter_names.push((ParameterKey::Positional, variable));
        parameter_names_internal.push(parameter.internal_name.clone());
    }

    for parameter in function.parameters.iter() {
        let variable = Variable::make_immutable(link_type(&parameter.param_type, scope));

        parameters.insert(Rc::clone(&variable));
        parameter_names.push((parameter.key.clone(), variable));
        parameter_names_internal.push(parameter.internal_name.clone());
    }

    return Rc::new(FunctionPointer {
        function_id: Uuid::new_v4(),
        pointer_id: Uuid::new_v4(),

        machine_interface: Rc::new(MachineFunctionInterface {
            parameters,
            return_type,
            injectable_pointers: requirements.iter()
                .flat_map(|x| x.functions_pointers.iter().map(|x| Rc::clone(&x.1)))
                .collect()
        }),
        human_interface: Rc::new(HumanFunctionInterface {
            name: function.identifier.clone(),
            alphanumeric_name: function.identifier.clone(),

            parameter_names,
            parameter_names_internal,

            requirements,
            form: if function.target.is_none() { FunctionForm::Global } else { FunctionForm::Member },
        }),
    });
}

pub fn link_operator_pointer(function: &abstract_syntax::Operator, parser_scope: &parser::scopes::Level, scope: &scopes::Hierarchy, requirements: HashSet<Rc<TraitConformanceRequirement>>) -> Rc<FunctionPointer> {
    let return_type = function.return_type.as_ref().map(|x| link_type(&x, scope));

    let mut parameters: HashSet<Rc<Variable>> = HashSet::new();
    let mut parameter_names: Vec<(ParameterKey, Rc<Variable>)> = vec![];
    let mut parameter_names_internal: Vec<String> = vec![];

    for parameter in function.lhs.iter().chain([&function.rhs]) {
        let variable = Variable::make_immutable(link_type(&parameter.param_type, scope));

        parameters.insert(Rc::clone(&variable));
        parameter_names.push((ParameterKey::Positional, variable));
        parameter_names_internal.push(parameter.internal_name.clone());
    }

    let is_binary = function.lhs.is_some();
    let pattern = parser_scope.resolve_operator_pattern(&function.operator, is_binary);

    return Rc::new(FunctionPointer {
        function_id: Uuid::new_v4(),
        pointer_id: Uuid::new_v4(),

        machine_interface: Rc::new(MachineFunctionInterface {
            parameters,
            return_type,
            injectable_pointers: requirements.iter()
                .flat_map(|x| x.functions_pointers.iter().map(|x| Rc::clone(&x.1)))
                .collect()
        }),
        human_interface: Rc::new(HumanFunctionInterface {
            name: function.operator.clone(),
            alphanumeric_name: pattern.alias.clone(),
            parameter_names,
            parameter_names_internal,

            requirements,
            form: FunctionForm::Operator,
        }),
    });
}

pub fn link_type(syntax: &abstract_syntax::TypeDeclaration, scope: &scopes::Hierarchy) -> Box<Type> {
    match syntax {
        abstract_syntax::TypeDeclaration::Identifier(id) => {
            scope.resolve_metatype(scopes::Environment::Global, id).clone()
        },
        abstract_syntax::TypeDeclaration::Monad { unit, shape } => {
            Box::new(Type {
                unit: TypeUnit::Monad,
                arguments: vec![link_type(&unit, scope)]
            })
        }
    }
}

pub fn link_specialized_type(syntax: &abstract_syntax::SpecializedType, scope: &scopes::Hierarchy) -> Box<Type> {
    Box::new(Type {
        unit: scope.resolve_metatype(scopes::Environment::Global, &syntax.unit).unit.clone(),
        arguments: syntax.elements.iter()
            .flat_map(|x| x)
            .map(|x| link_specialized_type(x, scope))
            .collect(),
    })
}
