use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;
use crate::semantic_analysis::computation_tree::{Type, Variable, VariableHome::*};

pub struct TenLangBuiltins {
    pub operators: TenLangBuiltinOperators
}

pub struct TenLangBuiltinOperators {
    pub add: Rc<Variable>,
    pub subtract: Rc<Variable>,
    pub multiply: Rc<Variable>,
    pub divide: Rc<Variable>,
}

pub fn create_builtins() -> (TenLangBuiltins, HashMap<String, Rc<Variable>>) {
    let mut globals: HashMap<String, Rc<Variable>> = HashMap::new();

    let mut new_global = |name: &str, type_id: &str| -> Rc<Variable> {
        let var = Rc::new(Variable {
            id: Uuid::new_v4(),
            home: Global,
            name: String::from(name),
            type_declaration: Box::new(Type::Identifier(String::from(type_id)))
        });
        globals.insert(var.name.clone(), Rc::clone(&var));
        return var
    };

    return (
        TenLangBuiltins {
            operators: TenLangBuiltinOperators {
                add: new_global("+", "Function"),
                subtract: new_global("-", "Function"),
                multiply: new_global("*", "Function"),
                divide: new_global("/", "Function"),
            }
        },
        globals
    )
}
