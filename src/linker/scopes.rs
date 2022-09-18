use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use uuid::Uuid;
use crate::program::functions::{FunctionForm, FunctionPointer, HumanFunctionInterface};
use crate::program::traits::{Trait, TraitConformanceDeclaration, TraitConformanceDeclarations, TraitConformanceRequirement};
use crate::program::generics::GenericMapping;
use crate::program::types::{Mutability, ParameterKey, Type, TypeUnit, Variable};

type VariablePool = HashMap<String, HashSet<Rc<Variable>>>;

#[derive(Copy, Clone, PartialEq)]
pub enum Environment {
    Global,
    Member
}

pub struct Level {
    pub global: VariablePool,
    pub member: VariablePool,
    pub trait_conformance_declarations: TraitConformanceDeclarations,
}

impl Level {
    pub fn new() -> Level {
        Level {
            global: HashMap::new(),
            member: HashMap::new(),
            trait_conformance_declarations: TraitConformanceDeclarations::new()
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

        let variable = Variable::make_immutable(Type::unit(TypeUnit::Function(Rc::clone(fun))));

        let mut variables = self.variables_mut(environment);

        if let Some(existing) = variables.get_mut(&fun.human_interface.name) {
            let existing_var = existing.iter().next().unwrap();

            if let TypeUnit::Function(_) = existing_var.type_declaration.as_ref().unit {
                existing.insert(variable);
            }
            else {
                panic!("Cannot overload with function '{}' if a variable exists in the same scope under the same name.", &fun.human_interface.name);
            }
        }
        else {
            variables.insert(fun.human_interface.name.clone(), HashSet::from([variable]));
        }
    }

    pub fn add_trait(&mut self, t: &Rc<Trait>) {
        let name = t.name.clone();
        self.insert_singleton(
            Environment::Global,
            Variable::make_immutable(Type::unit(TypeUnit::Trait(Rc::clone(t)))),
            &name
        );
    }

    pub fn insert_singleton(&mut self, environment: Environment, variable: Rc<Variable>, name: &String) {
        let mut variables = self.variables_mut(environment);

        if let Some(_) = variables.insert(name.clone(), HashSet::from([variable])) {
            panic!("Multiple variables of the same name: {}", name);
        }
    }

    pub fn push_variable(&mut self, environment: Environment, variable: Rc<Variable>, name: &String) {
        let mut variables = self.variables_mut(environment);

        variables.insert(name.clone(), HashSet::from([variable]));
    }

    pub fn contains(&mut self, environment: Environment, name: &String) -> bool {
        self.variables(environment).contains_key(name)
    }
}

pub struct Hierarchy<'a> {
    pub levels: Vec<&'a Level>,
    pub trait_conformance_declarations: TraitConformanceDeclarations,
}

impl <'a> Hierarchy<'a> {
    pub fn subscope(&self, new_scope: &'a Level) -> Hierarchy<'a> {
        let mut levels: Vec<&'a Level> = Vec::new();

        levels.push(new_scope);
        levels.extend(self.levels.iter());

        Hierarchy {
            levels,
            trait_conformance_declarations: TraitConformanceDeclarations::merge(&self.trait_conformance_declarations, &new_scope.trait_conformance_declarations)
        }
    }

    pub fn resolve_functions(&self, environment: Environment, variable_name: &String) -> Vec<&'a Rc<FunctionPointer>> {
        self.resolve(environment, variable_name).iter().map(|x|
            match &x.type_declaration.unit {
                TypeUnit::Function(function) => function,
                _ => panic!("{} is not a function.", variable_name)
            }
        ).collect()
    }

    pub fn resolve_metatype(&self, environment: Environment, variable_name: &String) -> &'a Box<Type> {
        let type_declaration = &self.resolve_unambiguous(environment, variable_name).type_declaration;

        match &type_declaration.unit {
            TypeUnit::MetaType => type_declaration.arguments.get(0).unwrap(),
            _ => panic!("{}' is not a type.", variable_name)
        }
    }

    pub fn resolve_trait(&self, environment: Environment, variable_name: &String) -> &'a Rc<Trait> {
        let type_declaration = &self.resolve_unambiguous(environment, variable_name).type_declaration;

        match &type_declaration.unit {
            TypeUnit::Trait(t) => t,
            _ => panic!("{}' is not a type.", variable_name)
        }
    }

    pub fn resolve_unambiguous(&self, environment: Environment, variable_name: &String) -> &'a Rc<Variable> {
        let matches = self.resolve(environment, variable_name);

        if matches.len() == 1 {
            matches.iter().next().unwrap()
        }
        else {
            panic!("Variable ambiguous: {}", variable_name);
        }
    }

    pub fn resolve(&self, environment: Environment, variable_name: &String) -> &'a HashSet<Rc<Variable>> {
        for scope in self.levels.iter() {
            if let Some(matches) = scope.variables(environment).get(variable_name) {
                return matches
            }
        }

        panic!("Variable '{}' could not be resolved", variable_name)
    }
}
