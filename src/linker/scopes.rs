use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::program::types::{FunctionInterface, PassedArgumentType, Type, Variable};

type VariablePool = Box<HashMap<String, HashSet<Rc<Variable>>>>;

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
            global: Box::new(HashMap::new()),
            member: Box::new(HashMap::new()),
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

    pub fn as_global_scope(&self) -> Hierarchy {
        Hierarchy {
            levels: vec![self]
        }
    }

    pub fn add_function(&mut self, environment: Environment, variable: Rc<Variable>) {
        let mut variables = self.variables_mut(environment);

        if let Some(existing) = variables.get_mut(&variable.name) {
            let existing_var = existing.iter().next().unwrap();

            if let Type::Function(_) = existing_var.type_declaration.as_ref() {
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

    pub fn resolve_function(&self, environment: Environment, variable_name: &String, arguments: &Vec<PassedArgumentType>) -> &'a Rc<FunctionInterface> {
        let functions: Vec<&Rc<FunctionInterface>> = self.resolve_functions(environment, variable_name).into_iter()
            .filter(|x| Type::arguments_satisfy_function(arguments, x))
            .collect();

        if functions.len() == 0 {
            panic!("{} could not be resolved for the passed arguments: {:?}", variable_name, arguments)
        }
        else if functions.len() > 1 {
            panic!("{} is ambiguous for the passed arguments: {:?}", variable_name, arguments)
        }
        else {
            functions[0]
        }
    }

    pub fn resolve_functions(&self, environment: Environment, variable_name: &String) -> Vec<&'a Rc<FunctionInterface>> {
        self.resolve(environment, variable_name).iter().map(|x|
            match &x.type_declaration.as_ref() {
                Type::Function(function) => function,
                _ => panic!("{} is not a function.", variable_name)
            }
        ).collect()
    }

    pub fn resolve_metatype(&self, environment: Environment, variable_name: &String) -> &'a Box<Type> {
        match &self.resolve_unambiguous(environment, variable_name).type_declaration.as_ref() {
            Type::MetaType(metatype) => metatype,
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
