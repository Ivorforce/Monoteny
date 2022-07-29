use std::collections::HashMap;
use std::rc::Rc;
use crate::linker::computation_tree::Variable;

pub struct ScopedVariables<'a> {
    pub scopes: Vec<&'a HashMap<String, Rc<Variable>>>,
}

impl <'a> ScopedVariables<'a> {
    pub fn resolve(&self, variable_name: &String) -> Rc<Variable> {
        for scope in self.scopes.iter() {
            if let Some(variable) = scope.get(variable_name) {
                return variable.clone()
            }
        }

        panic!("Variable '{}' could not be resolved", variable_name)
    }

    pub fn subscope(&self, new_scope: &'a HashMap<String, Rc<Variable>>) -> ScopedVariables<'a> {
        let mut scopes: Vec<&'a HashMap<String, Rc<Variable>>> = Vec::new();

        scopes.push(new_scope);
        scopes.extend(self.scopes.iter());

        ScopedVariables { scopes }
    }
}
