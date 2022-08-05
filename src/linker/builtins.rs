use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use uuid::Uuid;
use strum::{ IntoEnumIterator };
use crate::abstract_syntax::Mutability;
use crate::linker::computation_tree::*;
use crate::linker::primitives;
use crate::linker::scopes;

pub struct TenLangBuiltins {
    pub operators: TenLangBuiltinOperators,
    pub functions: TenLangBuiltinFunctions,
    pub primitive_metatypes: HashMap<primitives::Type, Box<Type>>,
    pub structs: TenLangBuiltinStructs,

    pub global_constants: scopes::Level,
}

pub struct TenLangBuiltinOperators {
    pub and: Rc<FunctionInterface>,
    pub or: Rc<FunctionInterface>,

    pub equal_to: Vec<Rc<FunctionInterface>>,
    pub not_equal_to: Vec<Rc<FunctionInterface>>,

    pub greater_than: Vec<Rc<FunctionInterface>>,
    pub greater_than_or_equal_to: Vec<Rc<FunctionInterface>>,
    pub lesser_than: Vec<Rc<FunctionInterface>>,
    pub lesser_than_or_equal_to: Vec<Rc<FunctionInterface>>,

    pub add: Vec<Rc<FunctionInterface>>,
    pub subtract: Vec<Rc<FunctionInterface>>,
    pub multiply: Vec<Rc<FunctionInterface>>,
    pub divide: Vec<Rc<FunctionInterface>>,
    pub exponentiate: Vec<Rc<FunctionInterface>>,
    pub modulo: Vec<Rc<FunctionInterface>>,

    pub positive: Vec<Rc<FunctionInterface>>,
    pub negative: Vec<Rc<FunctionInterface>>,
    pub not: Rc<FunctionInterface>,
}

pub struct TenLangBuiltinFunctions {
    pub print: Rc<FunctionInterface>,
}

#[allow(non_snake_case)]
pub struct TenLangBuiltinStructs {
    pub String: Rc<Struct>,
}

pub fn create_function_variable(interface: &Rc<FunctionInterface>) -> Rc<Variable> {
    Rc::new(Variable {
        id: Uuid::new_v4(),
        name: interface.name.clone(),
        type_declaration: Box::new(Type::Function(Rc::clone(interface))),
        mutability: Mutability::Immutable,
    })
}

pub fn create_same_parameters(declared_type: &Box<Type>, names: Vec<&str>) -> Vec<Box<Parameter>> {
    names.iter().enumerate()
        .map(|(idx, name)|
            Box::new(Parameter {
                external_key: ParameterKey::Int(idx as i32),
                variable: Rc::new(Variable {
                    id: Uuid::new_v4(),
                    name: String::from(*name),
                    type_declaration: declared_type.clone(),
                    mutability: Mutability::Immutable,
                }),
            })
        )
        .collect()
}

pub fn create_builtins() -> Rc<TenLangBuiltins> {
    let mut constants: scopes::Level = scopes::Level::new();

    let bool_type = Box::new(Type::Primitive(primitives::Type::Bool));

    // TODO Can we somehow change it so constants is not passed explicitly?
    //  It seems that every use in other functions borrows this function as mutable, so that the
    //  reference can be accessed from there. Only one mutable borrow can exist, so two functions
    //  calling this one is impossible...?
    let add_function = |constants: &mut scopes::Level, interface: Rc<FunctionInterface>| -> Rc<FunctionInterface> {
        constants.add_function(scopes::Environment::Exposed, create_function_variable(&interface));
        interface
    };

    // For now it's ok to assume the shape_returns types to be equal
    let add_binary_aa_x = |constants: &mut scopes::Level, name: &str, parameters: &Vec<Box<Type>>, return_type: &Box<Type>| -> Vec<Rc<FunctionInterface>> {
        parameters.iter().map(|x| {
            add_function(constants, Rc::new(FunctionInterface {
                id: Uuid::new_v4(),
                name: String::from(name),
                is_member_function: false,
                parameters: create_same_parameters(
                    x,
                    vec!["lhs", "rhs"]
                ),
                return_type: Some(return_type.clone()),
                generics: vec![],
            }))
        }).collect()
    };

    let add_binary_aa_a = |constants: &mut scopes::Level, name: &str, parameters: &Vec<Box<Type>>| -> Vec<Rc<FunctionInterface>> {
        parameters.iter().map(|x| {
            add_function(constants, Rc::new(FunctionInterface {
                id: Uuid::new_v4(),
                name: String::from(name),
                is_member_function: false,
                parameters: create_same_parameters(
                    x,
                    vec!["lhs", "rhs"]
                ),
                return_type: Some(x.clone()),
                generics: vec![],
            }))
        }).collect()
    };

    let add_binary_xx_x = |constants: &mut scopes::Level, name: &str, declared_type: &Box<Type>| -> Rc<FunctionInterface> {
        add_function(constants, Rc::new(FunctionInterface {
            id: Uuid::new_v4(),
            is_member_function: false,
            name: String::from(name),
            parameters: create_same_parameters(
                &declared_type,
                vec!["lhs", "rhs"]
            ),
            return_type: Some(declared_type.clone()),
            generics: vec![],
        }))
    };

    let add_unary_a_a = |constants: &mut scopes::Level, name: &str, parameters: &Vec<Box<Type>>| -> Vec<Rc<FunctionInterface>> {
        parameters.iter().map(|x| {
            add_function(constants, Rc::new(FunctionInterface {
                id: Uuid::new_v4(),
                name: String::from(name),
                is_member_function: false,
                parameters: create_same_parameters(
                    x,
                    vec!["value"]
                ),
                return_type: Some(x.clone()),
                generics: vec![],
            }))
        }).collect()
    };

    let primitive_metatypes = primitives::Type::iter()
        .map(|x| (x, Box::new(Type::MetaType(Box::new(Type::Primitive(x))))))
        .collect::<HashMap<primitives::Type, Box<Type>>>();

    for (primitive_type, metatype) in &primitive_metatypes {
        constants.insert_singleton(scopes::Environment::Exposed, Rc::new(Variable {
            id: Uuid::new_v4(),
            name: primitive_type.identifier_string(),
            type_declaration: metatype.clone(),
            mutability: Mutability::Immutable
        }));
    }

    let add_struct = |constants: &mut scopes::Level, name: &str| -> Rc<Struct> {
        let s = Rc::new(Struct {
            id: Uuid::new_v4(),
            name: String::from(name),
        });
        let s_type = Box::new(Type::MetaType(Box::new(Type::Struct(Rc::clone(&s)))));

        constants.insert_singleton(scopes::Environment::Exposed, Rc::new(Variable {
            id: Uuid::new_v4(),
            name: s.name.clone(),
            type_declaration: s_type,
            mutability: Mutability::Immutable,
        }));

        s
    };

    let all_primitives: Vec<Box<Type>> = primitives::Type::iter()
        .map(|x| Box::new(Type::Primitive(x)))
        .collect();
    let number_primitives: Vec<Box<Type>> = primitives::Type::NUMBERS.iter()
        .map(|x| Box::new(Type::Primitive(*x)))
        .collect();

    Rc::new(TenLangBuiltins {
        operators: TenLangBuiltinOperators {
            and: add_binary_xx_x(&mut constants, "&&", &bool_type),
            or: add_binary_xx_x(&mut constants, "||", &bool_type),

            // These are n-ary in syntax but binary in implementation.
            equal_to: add_binary_aa_x(&mut constants, "==", &all_primitives, &bool_type),
            not_equal_to: add_binary_aa_x(&mut constants, "!=", &all_primitives, &bool_type),

            greater_than: add_binary_aa_x(&mut constants, ">", &number_primitives, &bool_type),
            greater_than_or_equal_to: add_binary_aa_x(&mut constants, ">=", &number_primitives, &bool_type),
            lesser_than: add_binary_aa_x(&mut constants, "<", &number_primitives, &bool_type),
            lesser_than_or_equal_to: add_binary_aa_x(&mut constants, "<=", &number_primitives, &bool_type),

            add: add_binary_aa_a(&mut constants, "+", &number_primitives),
            subtract: add_binary_aa_a(&mut constants, "-", &number_primitives),
            multiply: add_binary_aa_a(&mut constants, "*", &number_primitives),
            divide: add_binary_aa_a(&mut constants, "/", &number_primitives),
            exponentiate: add_binary_aa_a(&mut constants, "**", &number_primitives),
            modulo: add_binary_aa_a(&mut constants, "%", &number_primitives),

            positive: add_unary_a_a(&mut constants, "+", &number_primitives),
            negative: add_unary_a_a(&mut constants, "-", &number_primitives),
            not: add_unary_a_a(&mut constants, "!", &vec![bool_type]).into_iter().next().unwrap(),
        },
        functions: TenLangBuiltinFunctions {
            print: add_function(&mut constants, Rc::new(FunctionInterface {
                id: Uuid::new_v4(),
                is_member_function: false,
                name: String::from("print"),
                parameters: create_same_parameters(&Type::make_any(), vec!["object"]),
                generics: vec![],
                return_type: None
            })),
        },
        structs: TenLangBuiltinStructs {
            String: add_struct(&mut constants, "String")
        },
        primitive_metatypes,
        global_constants: constants,
    })
}
