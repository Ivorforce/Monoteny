use std::collections::HashMap;
use std::rc::Rc;
use crate::linker::builtins::TenLangBuiltins;
use crate::linker::computation_tree::{FunctionInterface, Type, Variable};

pub struct Scope<'a> {
    pub scopes: Vec<&'a HashMap<String, Rc<Variable>>>,
}

impl <'a> Scope<'a> {
    pub fn resolve_static_fn(&self, variable_name: &String) -> &'a Rc<FunctionInterface> {
        let variable = self.resolve(variable_name);

        match &variable.type_declaration.as_ref() {
            Type::Function(function) => function,
            _ => panic!("{} resolved to a non-function of type '{:?}', which should be impossible.", variable_name, variable.type_declaration)
        }
    }

    pub fn resolve(&self, variable_name: &String) -> &'a Rc<Variable> {
        for scope in self.scopes.iter() {
            if let Some(variable) = scope.get(variable_name) {
                return variable
            }
        }

        panic!("Variable '{}' could not be resolved", variable_name)
    }

    pub fn subscope(&self, new_scope: &'a HashMap<String, Rc<Variable>>) -> Scope<'a> {
        let mut scopes: Vec<&'a HashMap<String, Rc<Variable>>> = Vec::new();

        scopes.push(new_scope);
        scopes.extend(self.scopes.iter());

        Scope { scopes }
    }
}
