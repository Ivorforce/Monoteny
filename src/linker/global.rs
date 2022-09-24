use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use guard::guard;
use itertools::{Itertools, zip_eq};
use uuid::Uuid;

use crate::parser;
use crate::parser::abstract_syntax;
use crate::program::computation_tree::*;
use crate::linker::imperative::ImperativeLinker;
use crate::linker::scopes;
use crate::program::traits::{Trait, TraitConformanceDeclaration, TraitConformanceRequirement, TraitConformanceScope};
use crate::program::{primitives, Program};
use crate::program::allocation::Variable;
use crate::program::builtins::*;
use crate::program::functions::{FunctionForm, FunctionPointer, FunctionPointerTarget, HumanFunctionInterface, MachineFunctionInterface, ParameterKey};
use crate::program::generics::GenericMapping;
use crate::program::global::{FunctionImplementation, GlobalStatement};
use crate::program::types::*;
use crate::util::multimap::extend_multimap;


struct GlobalLinker<'a> {
    functions: Vec<FunctionWithoutBody<'a>>,
    traits: HashSet<Rc<Trait>>,
    global_variables: scopes::Level,
    parser_scope: &'a parser::scopes::Level,
    builtins: &'a TenLangBuiltins,
}

struct FunctionWithoutBody<'a> {
    pointer: Rc<FunctionPointer>,
    body: &'a Vec<Box<abstract_syntax::Statement>>,
    conformance_delegations: HashMap<Rc<TraitConformanceRequirement>, Rc<TraitConformanceDeclaration>>,
}

pub fn link_file(syntax: abstract_syntax::Program, parser_scope: &parser::scopes::Level, scope: &scopes::Hierarchy, builtins: &TenLangBuiltins) -> Program {
    let mut global_linker = GlobalLinker {
        functions: Vec::new(),
        traits: HashSet::new(),
        global_variables: scopes::Level::new(),
        parser_scope,
        builtins
    };

    // Alias global names
    // TODO

    // Resolve global types / interfaces
    for statement in &syntax.global_statements {
        global_linker.link_global_statement(statement.as_ref(), scope, &HashMap::new());
    }

    let global_variable_scope = scope.subscope(&global_linker.global_variables);
    let mut global_statements = vec![];
    let mut functions: HashSet<Rc<FunctionImplementation>> = HashSet::new();

    // Resolve function bodies
    for fun in global_linker.functions.iter() {
        let mut variable_names = HashMap::new();
        for (name, (_, variable)) in zip_eq(fun.pointer.human_interface.parameter_names_internal.iter(), fun.pointer.human_interface.parameter_names.iter()) {
            variable_names.insert(Rc::clone(variable), name.clone());
        }

        // TODO Inject traits, not pointers
        let mut resolver = Box::new(ImperativeLinker {
            function: Rc::clone(&fun.pointer),
            builtins,
            generics: GenericMapping::new(),
            variable_names,
            conformance_delegations: &fun.conformance_delegations,
        });

        // TODO Maybe we should just re-use the whole scope level instead of doing this manually
        //  ... but only from > file level, i.e. scopes. Can't do that without trait shadowing though.
        let mut injection_level = scopes::Level::new();

        // Add the local declarations (derived from requirements) to the scope
        for declaration in fun.conformance_delegations.values() {
            injection_level.add_trait_conformance(declaration);
        }

        let function_scope = global_variable_scope.subscope(&injection_level);

        let implementation = resolver.link_function_body(fun.body, &function_scope);
        functions.insert(Rc::clone(&implementation));
        global_statements.push(GlobalStatement::Function(implementation));
    }

    let main_function = functions.iter()
        .filter(|f| {
            f.human_interface.name == "main"
            && f.human_interface.form == FunctionForm::Global
            && f.human_interface.parameter_names.is_empty()
        })
        .map(Rc::clone)
        .next();

    return Program {
        functions,
        traits: global_linker.traits.iter().map(Rc::clone).collect(),
        global_statements,
        main_function,
    }
}

impl <'a> GlobalLinker<'a> {
    pub fn link_global_statement(&mut self, statement: &'a abstract_syntax::GlobalStatement, scope: &scopes::Hierarchy, conformance_delegations: &HashMap<Rc<TraitConformanceRequirement>, Rc<TraitConformanceDeclaration>>) {
        match statement {
            abstract_syntax::GlobalStatement::Scope(syntax) => {
                with_delegations(scope, conformance_delegations, syntax.requirements.iter().flat_map(|x| x.iter()).map(|x| x.as_ref()), |scope, conformance_delegations| {
                    for statement in &syntax.statements {
                        self.link_global_statement(statement.as_ref(), scope, conformance_delegations);
                    }
                });
            }
            abstract_syntax::GlobalStatement::FunctionDeclaration(syntax) => {
                with_anonymous_generics(&syntax.gather_type_names(), scope, conformance_delegations, |scope, conformance_delegations| {
                    let fun = link_function_pointer(&syntax, scope, conformance_delegations);

                    self.functions.push(FunctionWithoutBody {
                        pointer: Rc::clone(&fun),
                        body: &syntax.body,
                        conformance_delegations: conformance_delegations.clone()
                    });

                    // Create a variable for the function
                    self.global_variables.add_function(&fun);

                    // if interface.is_member_function {
                    // TODO Create an additional variable as Metatype.function...?
                    // }
                });
            }
            abstract_syntax::GlobalStatement::Operator(syntax) => {
                with_anonymous_generics(&syntax.gather_type_names(), scope, conformance_delegations, |scope, conformance_delegations| {
                    let fun = link_operator_pointer(&syntax, self.parser_scope, scope, conformance_delegations);

                    self.functions.push(FunctionWithoutBody {
                        pointer: Rc::clone(&fun),
                        body: &syntax.body,
                        conformance_delegations: conformance_delegations.clone()
                    });

                    // Create a variable for the function
                    self.global_variables.add_function(&fun);
                });
            }
            _ => {}
        }
    }
}

pub fn with_anonymous_generics<'a, F>(type_names: &'a Vec<&'a String>, scope: &scopes::Hierarchy, conformance_delegations: &HashMap<Rc<TraitConformanceRequirement>, Rc<TraitConformanceDeclaration>>, fun: F) where F: FnOnce(&scopes::Hierarchy, &HashMap<Rc<TraitConformanceRequirement>, Rc<TraitConformanceDeclaration>>) {
    let mut level = Box::new(scopes::Level::new());
    let mut requirements_syntax = Vec::new();

    let mut needs_scope = false;
    let mut needs_requirements = false;

    for type_name in type_names {
        if type_name.starts_with("#") && !level.contains(scopes::Environment::Global, type_name) {
            level.insert_singleton(scopes::Environment::Global, Variable::make_immutable(Type::meta(Type::make_any())), type_name);
            needs_scope = true;
        }
        else if type_name.starts_with("$") && !level.contains(scopes::Environment::Global, type_name) {
            level.insert_singleton(scopes::Environment::Global, Variable::make_immutable(Type::meta(Type::make_any())), type_name);
            needs_scope = true;

            let trait_name = String::from(&type_name[1..]);

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
            with_delegations(&subscope, conformance_delegations, requirements_syntax.iter(), fun);
        }
        else {
            fun(&subscope, conformance_delegations);
        }
    }
    else {
        fun(&scope, conformance_delegations);
    }
}

pub fn with_delegations<'a, I, F>(scope: &scopes::Hierarchy, conformance_delegations: &HashMap<Rc<TraitConformanceRequirement>, Rc<TraitConformanceDeclaration>>, requirements_syntax: I, fun: F) where F: FnOnce(&scopes::Hierarchy, &HashMap<Rc<TraitConformanceRequirement>, Rc<TraitConformanceDeclaration>>), I: Iterator<Item=&'a abstract_syntax::TraitDeclaration> {
    let mut conformance_delegations = conformance_delegations.clone();
    let mut level_with_requirements = scopes::Level::new();

    for requirement_syntax in requirements_syntax {
        let trait_ = Rc::clone(scope.resolve_trait(scopes::Environment::Global, &requirement_syntax.unit));
        let arguments: Vec<Box<Type>> = requirement_syntax.elements.iter().map(|x| {
            link_specialized_type(x, &scope)
        }).collect();

        let requirement = Trait::require(&trait_, arguments.clone());
        let declaration = Trait::assume_granted(&trait_, arguments);

        for pointer in declaration.function_implementations.values() {
            level_with_requirements.add_function(pointer);
        }

        conformance_delegations.insert(requirement, declaration);
    }

    let subscope = scope.subscope(&level_with_requirements);

    fun(&subscope, &conformance_delegations);
}

pub fn link_function_pointer(function: &abstract_syntax::Function, scope: &scopes::Hierarchy, conformance_delegations: &HashMap<Rc<TraitConformanceRequirement>, Rc<TraitConformanceDeclaration>>) -> Rc<FunctionPointer> {
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
        pointer_id: Uuid::new_v4(),
        target: FunctionPointerTarget::Static { implementation_id: Uuid::new_v4() },

        machine_interface: Rc::new(MachineFunctionInterface {
            parameters,
            return_type,
            requirements: conformance_delegations.keys().map(Rc::clone).collect()
        }),
        human_interface: Rc::new(HumanFunctionInterface {
            name: function.identifier.clone(),
            alphanumeric_name: function.identifier.clone(),

            parameter_names,
            parameter_names_internal,

            form: if function.target.is_none() { FunctionForm::Global } else { FunctionForm::Member },
        }),
    });
}

pub fn link_operator_pointer(function: &abstract_syntax::Operator, parser_scope: &parser::scopes::Level, scope: &scopes::Hierarchy, conformance_delegations: &HashMap<Rc<TraitConformanceRequirement>, Rc<TraitConformanceDeclaration>>) -> Rc<FunctionPointer> {
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
        pointer_id: Uuid::new_v4(),
        target: FunctionPointerTarget::Static { implementation_id: Uuid::new_v4() },

        machine_interface: Rc::new(MachineFunctionInterface {
            parameters,
            return_type,
            requirements: conformance_delegations.keys().map(Rc::clone).collect()
        }),
        human_interface: Rc::new(HumanFunctionInterface {
            name: function.operator.clone(),
            alphanumeric_name: pattern.alias.clone(),
            parameter_names,
            parameter_names_internal,

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
