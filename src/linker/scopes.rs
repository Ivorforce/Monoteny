use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::linker::builtins::TenLangBuiltins;
use crate::linker::computation_tree::{FunctionInterface, PassedArgument, PassedArgumentType, Type, Variable};

pub struct ScopeLevel {
    variables: Box<HashMap<String, HashSet<Rc<Variable>>>>
}

impl ScopeLevel {
    pub fn new() -> ScopeLevel {
        ScopeLevel {
            variables: Box::new(HashMap::new())
        }
    }

    pub fn as_global_scope(&self) -> Scope {
        Scope {
            scopes: vec![self]
        }
    }

    pub fn add_function(&mut self, variable: Rc<Variable>) {
        if let Some(existing) = self.variables.get_mut(&variable.name) {
            let existing_var = existing.iter().next().unwrap();

            if let Type::Function(_) = existing_var.type_declaration.as_ref() {
                existing.insert(variable);
            }
            else {
                panic!("Cannot overload with function '{}' if a variable exists in the same scope under the same name.", &variable.name);
            }
        }
        else {
            self.variables.insert(variable.name.clone(), HashSet::from([variable]));
        }
    }

    pub fn insert_singleton(&mut self, variable: Rc<Variable>) {
        if let Some(existing) = self.variables.insert(variable.name.clone(), HashSet::from([variable])) {
            panic!("Multiple variables of the same name: {}", existing.iter().next().unwrap().name);
        }
    }

    pub fn push_variable(&mut self, variable: Rc<Variable>) {
        self.variables.insert(variable.name.clone(), HashSet::from([variable]));
    }
}

pub struct Scope<'a> {
    scopes: Vec<&'a ScopeLevel>
}

impl <'a> Scope<'a> {
    pub fn subscope(&self, new_scope: &'a ScopeLevel) -> Scope<'a> {
        let mut scopes: Vec<&'a ScopeLevel> = Vec::new();

        scopes.push(new_scope);
        scopes.extend(self.scopes.iter());

        Scope { scopes }
    }

    pub fn resolve_function(&self, variable_name: &String, arguments: &Vec<PassedArgumentType>) -> &'a Rc<FunctionInterface> {
        let functions: Vec<&Rc<FunctionInterface>> = self.resolve_functions(variable_name).into_iter()
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

    pub fn resolve_functions(&self, variable_name: &String) -> Vec<&'a Rc<FunctionInterface>> {
        self.resolve(variable_name).iter().map(|x|
            match &x.type_declaration.as_ref() {
                Type::Function(function) => function,
                _ => panic!("{} is not a function.", variable_name)
            }
        ).collect()
    }

    pub fn resolve_metatype(&self, variable_name: &String) -> &'a Box<Type> {
        match &self.resolve_unambiguous(variable_name).type_declaration.as_ref() {
            Type::MetaType(metatype) => metatype,
            _ => panic!("{}' is not a type.", variable_name)
        }
    }

    pub fn resolve_unambiguous(&self, variable_name: &String) -> &'a Rc<Variable> {
        let matches = self.resolve(variable_name);

        if matches.len() == 1 {
            matches.iter().next().unwrap()
        }
        else {
            panic!("Variable ambiguous: {}", variable_name);
        }
    }

    pub fn resolve(&self, variable_name: &String) -> &'a HashSet<Rc<Variable>> {
        for scope in self.scopes.iter() {
            if let Some(matches) = scope.variables.get(variable_name) {
                return matches
            }
        }

        panic!("Variable '{}' could not be resolved", variable_name)
    }
}
