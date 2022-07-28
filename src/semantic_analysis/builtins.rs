use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;
use crate::semantic_analysis::computation_tree::{Function, Parameter, ParameterKey, Type, Variable, VariableHome::*};

pub struct TenLangBuiltins {
    pub operators: TenLangBuiltinOperators,
    pub functions: TenLangBuiltinFunctions
}

pub struct TenLangBuiltinOperators {
    pub add: Rc<Function>,
    pub subtract: Rc<Function>,
    pub multiply: Rc<Function>,
    pub divide: Rc<Function>,
}

pub struct TenLangBuiltinFunctions {
    pub print: Rc<Function>
}

pub fn create_builtins() -> (TenLangBuiltins, HashMap<String, Rc<Variable>>) {
    let mut constants: HashMap<String, Rc<Variable>> = HashMap::new();

    let mut add_function = |name: &str, parameters: Vec<Box<Parameter>>, return_type: Option<Box<Type>>| -> Rc<Function> {
        let function = Rc::new(Function {
            id: Uuid::new_v4(),
            name: String::from(name),
            return_type,
            variables: parameters.iter()
                .map(|x| (x.variable.id, x.variable.clone()))
                .collect(),
            parameters,
            statements: vec![]
        });

        let var = Rc::new(Variable {
            id: Uuid::new_v4(),
            home: Global,
            name: String::from(name),
            type_declaration: Box::new(Type::Function(function.clone()))
        });
        constants.insert(var.name.clone(), Rc::clone(&var));

        return function
    };

    // For now it's ok to assume the 3 types to be equal
    let mut add_binary_operator = |name: &str| -> Rc<Function> {
        let generic_type = Box::new(Type::Generic(Uuid::new_v4()));

        let parameters: Vec<Box<Parameter>> = vec!["lhs", "rhs"].iter()
            .map(|name| Box::new(Parameter {
                external_key: ParameterKey::Keyless,
                variable: Rc::new(Variable {
                    id: Uuid::new_v4(),
                    home: Local,
                    name: String::from(*name),
                    type_declaration: generic_type.clone()
                })
            })).collect();

        return add_function(name, parameters, Some(generic_type));
    };

    return (
        TenLangBuiltins {
            operators: TenLangBuiltinOperators {
                add: add_binary_operator("+"),
                subtract: add_binary_operator("-"),
                multiply: add_binary_operator("*"),
                divide: add_binary_operator("/"),
            },
            functions: TenLangBuiltinFunctions {
                print: add_function(
                    "print", vec![
                        Box::new(Parameter {
                            external_key: ParameterKey::Keyless,
                            variable: Rc::new(Variable {
                                id: Uuid::new_v4(),
                                home: Local,
                                name: String::from("object"),
                                type_declaration: Box::new(Type::Identifier(String::from("Any")))
                            })
                        })
                    ], None
                )
            }
        },
        constants
    )
}
