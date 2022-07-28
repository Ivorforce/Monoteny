use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;
use crate::abstract_syntax::Mutability;
use crate::linker::computation_tree::*;

pub struct TenLangBuiltins {
    pub operators: TenLangBuiltinOperators,
    pub functions: TenLangBuiltinFunctions
}

pub struct TenLangBuiltinOperators {
    pub add: Rc<FunctionInterface>,
    pub subtract: Rc<FunctionInterface>,
    pub multiply: Rc<FunctionInterface>,
    pub divide: Rc<FunctionInterface>,
}

pub struct TenLangBuiltinFunctions {
    pub print: Rc<FunctionInterface>
}

pub fn create_builtins() -> (TenLangBuiltins, HashMap<String, Rc<Variable>>) {
    let mut constants: HashMap<String, Rc<Variable>> = HashMap::new();

    let mut add_function = |name: &str, parameters: Vec<Box<Parameter>>, return_type: Option<Box<Type>>| -> Rc<FunctionInterface> {
        let interface = Rc::new(FunctionInterface {
            id: Uuid::new_v4(),
            name: String::from(name),
            return_type,
            parameters,
        });

        let var = Rc::new(Variable {
            id: Uuid::new_v4(),
            name: String::from(name),
            type_declaration: Box::new(Type::Function(interface.clone())),
            mutability: Mutability::Immutable,
        });
        constants.insert(var.name.clone(), Rc::clone(&var));

        return interface
    };

    // For now it's ok to assume the 3 types to be equal
    let mut add_binary_operator = |name: &str| -> Rc<FunctionInterface> {
        let generic_type = Box::new(Type::Generic(Uuid::new_v4()));

        let parameters: Vec<Box<Parameter>> = vec!["lhs", "rhs"].iter().enumerate()
            .map(|(idx, name)| Box::new(Parameter {
                external_key: ParameterKey::Int(idx as i32),
                variable: Rc::new(Variable {
                    id: Uuid::new_v4(),
                    name: String::from(*name),
                    type_declaration: generic_type.clone(),
                    mutability: Mutability::Immutable,
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
                            external_key: ParameterKey::Int(0),
                            variable: Rc::new(Variable {
                                id: Uuid::new_v4(),
                                name: String::from("object"),
                                type_declaration: Box::new(Type::Identifier(String::from("Any"))),
                                mutability: Mutability::Immutable,
                            })
                        })
                    ], None
                )
            }
        },
        constants
    )
}
