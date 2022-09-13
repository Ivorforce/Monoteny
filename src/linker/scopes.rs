use std::collections::{HashMap, HashSet};
use std::iter::zip;
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use uuid::Uuid;
use crate::program::generics::GenericMapping;
use crate::program::types::{FunctionInterface, PassedArgumentType, Type, TypeUnit, Variable};

type VariablePool = HashMap<String, HashSet<Rc<Variable>>>;

#[derive(Copy, Clone, PartialEq)]
pub enum Environment {
    Global,
    Member
}

pub struct Level {
    global: VariablePool,
    member: VariablePool,
}

impl Level {
    pub fn new() -> Level {
        Level {
            global: HashMap::new(),
            member: HashMap::new(),
        }
    }

    pub fn as_global_scope(&self) -> Hierarchy {
        Hierarchy {
            levels: vec![self]
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

    pub fn add_function(&mut self, environment: Environment, variable: Rc<Variable>) {
        let mut variables = self.variables_mut(environment);

        if let Some(existing) = variables.get_mut(&variable.name) {
            let existing_var = existing.iter().next().unwrap();

            if let TypeUnit::Function(_) = existing_var.type_declaration.as_ref().unit {
                existing.insert(variable);
            }
            else {
                panic!("Cannot overload with function '{}' if a variable exists in the same scope under the same name.", &variable.name);
            }
        }
        else {
            variables.insert(variable.name.clone(), HashSet::from([variable]));
        }
    }

    pub fn insert_singleton(&mut self, environment: Environment, variable: Rc<Variable>) {
        let mut variables = self.variables_mut(environment);

        if let Some(existing) = variables.insert(variable.name.clone(), HashSet::from([variable])) {
            panic!("Multiple variables of the same name: {}", existing.iter().next().unwrap().name);
        }
    }

    pub fn push_variable(&mut self, environment: Environment, variable: Rc<Variable>) {
        let mut variables = self.variables_mut(environment);

        variables.insert(variable.name.clone(), HashSet::from([variable]));
    }
}

pub struct Hierarchy<'a> {
    levels: Vec<&'a Level>
}

impl <'a> Hierarchy<'a> {
    pub fn subscope(&self, new_scope: &'a Level) -> Hierarchy<'a> {
        let mut levels: Vec<&'a Level> = Vec::new();

        levels.push(new_scope);
        levels.extend(self.levels.iter());

        Hierarchy { levels }
    }

    fn pair_arguments_to_parameters<'b>(arguments: &'b Vec<PassedArgumentType>, function: &'b FunctionInterface) -> Option<Vec<(&'b Type, &'b Type)>> {
        if arguments.len() != function.parameters.len() {
            return None;
        }

        return zip(arguments.iter(), function.parameters.iter()).map(|(arg, param)| {
            if &arg.key != &param.external_key {
                return None;
            }

            return Some((
                arg.value_type.as_deref().unwrap(),
                param.variable.type_declaration.as_ref()
            ))
        }).collect()
    }

    pub fn resolve_function(&self, environment: Environment, variable_name: &String, arguments: &Vec<PassedArgumentType>, mapping: &mut GenericMapping) -> &'a Rc<FunctionInterface> {
        for argument in arguments {
            if argument.value_type.is_none() {
                return panic!("Argument to function '{}({:?}: ?)' resolves to nothing.", variable_name, argument.key)
            }
        }

        let candidates: Vec<(&Rc<FunctionInterface>, Vec<(&Type, &Type)>)> = self.resolve_functions(environment, variable_name).into_iter()
            .flat_map(|x| Some((x, Hierarchy::pair_arguments_to_parameters(arguments, x)?)))
            .collect();

        if candidates.len() == 0 {
            panic!("No function could be found with signature {}({:?})", variable_name, arguments.iter().map(|x| &x.key))
        }

        let candidates: Vec<(&Rc<FunctionInterface>, Vec<(&Type, &Type)>)> = candidates.into_iter().flat_map(|(x, pairs)| {
            let mut clone: GenericMapping = mapping.clone();
            clone.merge_pairs(&pairs).ok()?;
            Some((x, pairs))
        }).collect();

        if candidates.len() == 0 {
            panic!("{} could not be resolved for the passed arguments: {:?}", variable_name, arguments)
        }
        else if candidates.len() > 1 {
            panic!("{} is ambiguous for the passed arguments: {:?}", variable_name, arguments)
        }
        else {
            // TODO This kinda ugly code
            let seed = Uuid::new_v4();
            let unique_params: Vec<(&Type, Box<Type>)> = candidates[0].1.iter()
                .map(|(arg, param)| (*arg, param.uniqueify(&seed)))
                .collect();

            // Actually bind the generics w.r.t. the selected function
            mapping.merge_pairs(
                &unique_params.iter().map(|(arg, param)| (*arg, param.as_ref())).collect()
            ).unwrap();
            candidates[0].0
        }
    }

    pub fn resolve_functions(&self, environment: Environment, variable_name: &String) -> Vec<&'a Rc<FunctionInterface>> {
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
