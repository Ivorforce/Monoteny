use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use uuid::Uuid;
use crate::program::allocation::{Mutability, Reference};
use crate::program::functions::{FunctionForm, FunctionPointer, HumanFunctionInterface, ParameterKey};
use crate::program::traits::{Trait, TraitConformanceDeclaration, TraitConformanceRequirement, TraitConformanceScope};
use crate::program::generics::TypeForest;
use crate::program::types::{TypeProto, TypeUnit};

// Note: While a single pool cannot own overloaded variables, multiple same-level pools (-> from imports) can.
// When we have imports, this should be ignored until referenced, to avoid unnecessary import complications.
// For these cases, we could store an AMBIGUOUS value inside our pool, crashing when accessed?
type VariablePool = HashMap<String, Rc<Reference>>;

#[derive(Copy, Clone, PartialEq)]
pub enum Environment {
    Global,
    Member
}

pub struct Level {
    pub global: VariablePool,
    pub member: VariablePool,
    pub trait_conformance_declarations: TraitConformanceScope,
}

impl Level {
    pub fn new() -> Level {
        Level {
            global: HashMap::new(),
            member: HashMap::new(),
            trait_conformance_declarations: TraitConformanceScope::new()
        }
    }

    pub fn as_global_scope(&self) -> Hierarchy {
        Hierarchy {
            levels: vec![self],
            trait_conformance_declarations: self.trait_conformance_declarations.clone()
        }
    }

    pub fn variables_mut(&mut self, environment: Environment) -> &mut VariablePool {
        match environment {
            Environment::Global => &mut self.global,
            Environment::Member => &mut self.member
        }
    }

    pub fn variables(&self, environment: Environment) -> &VariablePool {
        match environment {
            Environment::Global => &self.global,
            Environment::Member => &self.member
        }
    }

    pub fn add_function(&mut self, fun: &Rc<FunctionPointer>) {
        let environment = match fun.human_interface.form {
            FunctionForm::Member => Environment::Member,
            _ => Environment::Global
        };

        let mut variables = self.variables_mut(environment);

        // Remove the current FunctionOverload reference and replace with a reference containing also our new overload.
        // This may seem weird at first but it kinda makes sense - if someone queries the scope, gets a reference,
        // and then the scope is modified, the previous caller still expects their reference to not change.
        if let Some(existing) = variables.remove(&fun.human_interface.name) {
            if let TypeUnit::FunctionOverload(functions) = &existing.type_declaration.unit {
                let functions = functions.iter().chain([fun]).map(Rc::clone).collect();

                let variable = Reference::make_immutable(
                    TypeProto::unit(TypeUnit::FunctionOverload(functions))
                );

                variables.insert(fun.human_interface.name.clone(), variable);
            }
            else {
                panic!("Cannot overload with function '{}' if a variable exists in the same scope under the same name.", &fun.human_interface.name);
            }
        }
        else {
            let variable = Reference::make_immutable(
                TypeProto::unit(TypeUnit::FunctionOverload(HashSet::from([Rc::clone(fun)])))
            );

            variables.insert(fun.human_interface.name.clone(), variable);
        }
    }

    pub fn add_trait(&mut self, t: &Rc<Trait>) {
        let name = t.name.clone();
        self.insert_singleton(
            Environment::Global,
            Reference::make_immutable(TypeProto::unit(TypeUnit::Trait(Rc::clone(t)))),
            &name
        );
    }

    pub fn add_trait_conformance(&mut self, declaration: &Rc<TraitConformanceDeclaration>) {
        self.trait_conformance_declarations.add(declaration);
        for (_, pointer) in declaration.function_implementations.iter() {
            self.add_function(pointer);
        }
        for (_, declaration) in declaration.trait_requirements_conformance.iter() {
            self.add_trait_conformance(declaration);
        }
    }

    pub fn insert_singleton(&mut self, environment: Environment, variable: Rc<Reference>, name: &String) {
        let mut variables = self.variables_mut(environment);

        if let Some(_) = variables.insert(name.clone(), variable) {
            panic!("Multiple variables of the same name: {}", name);
        }
    }

    pub fn push_variable(&mut self, environment: Environment, variable: Rc<Reference>, name: &String) {
        let mut variables = self.variables_mut(environment);

        variables.insert(name.clone(), variable);
    }

    pub fn contains(&mut self, environment: Environment, name: &String) -> bool {
        self.variables(environment).contains_key(name)
    }
}

pub struct Hierarchy<'a> {
    pub levels: Vec<&'a Level>,
    pub trait_conformance_declarations: TraitConformanceScope,
}

impl <'a> Hierarchy<'a> {
    pub fn subscope(&self, new_scope: &'a Level) -> Hierarchy<'a> {
        let mut levels: Vec<&'a Level> = Vec::new();

        levels.push(new_scope);
        levels.extend(self.levels.iter());

        Hierarchy {
            levels,
            trait_conformance_declarations: TraitConformanceScope::merge(&self.trait_conformance_declarations, &new_scope.trait_conformance_declarations)
        }
    }

    pub fn resolve_functions(&self, environment: Environment, variable_name: &String) -> &'a HashSet<Rc<FunctionPointer>> {
        match &self.resolve(environment, variable_name).type_declaration.unit {
            TypeUnit::FunctionOverload(functions) => functions,
            _ => panic!("{} is not a function.", variable_name)
        }
    }

    pub fn resolve_metatype(&self, environment: Environment, variable_name: &String) -> &'a Box<TypeProto> {
        let type_declaration = &self.resolve(environment, variable_name).type_declaration;

        match &type_declaration.unit {
            TypeUnit::MetaType => type_declaration.arguments.get(0).unwrap(),
            _ => panic!("{}' is not a type.", variable_name)
        }
    }

    pub fn resolve_trait(&self, environment: Environment, variable_name: &String) -> &'a Rc<Trait> {
        let type_declaration = &self.resolve(environment, variable_name).type_declaration;

        match &type_declaration.unit {
            TypeUnit::Trait(t) => t,
            _ => panic!("{}' is not a type.", variable_name)
        }
    }

    pub fn resolve(&self, environment: Environment, variable_name: &String) -> &'a Rc<Reference> {
        for scope in self.levels.iter() {
            if let Some(matches) = scope.variables(environment).get(variable_name) {
                return matches
            }
        }

        panic!("Variable '{}' could not be resolved", variable_name)
    }
}
