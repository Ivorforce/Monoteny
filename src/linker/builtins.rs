use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;
use strum::{ IntoEnumIterator };
use crate::abstract_syntax::Mutability;
use crate::linker::computation_tree::*;
use crate::linker::primitives;
use crate::linker::scopes::Scope;

pub struct TenLangBuiltins {
    pub operators: TenLangBuiltinOperators,
    pub functions: TenLangBuiltinFunctions,
    pub primitive_metatypes: HashMap<primitives::Type, Box<Type>>,
    pub structs: TenLangBuiltinStructs,

    pub global_constants: Box<HashMap<String, Rc<Variable>>>,
}

pub struct TenLangBuiltinOperators {
    pub and: Rc<FunctionInterface>,
    pub or: Rc<FunctionInterface>,

    pub equal_to: Rc<FunctionInterface>,
    pub not_equal_to: Rc<FunctionInterface>,

    pub greater_than: Rc<FunctionInterface>,
    pub greater_than_or_equal_to: Rc<FunctionInterface>,
    pub lesser_than: Rc<FunctionInterface>,
    pub lesser_than_or_equal_to: Rc<FunctionInterface>,

    pub add: Rc<FunctionInterface>,
    pub subtract: Rc<FunctionInterface>,
    pub multiply: Rc<FunctionInterface>,
    pub divide: Rc<FunctionInterface>,
    pub exponentiate: Rc<FunctionInterface>,
    pub modulo: Rc<FunctionInterface>,

    pub positive: Rc<FunctionInterface>,
    pub negative: Rc<FunctionInterface>,
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
    vec!["lhs", "rhs"].iter().enumerate()
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
    let mut constants: Box<HashMap<String, Rc<Variable>>> = Box::new(HashMap::new());

    let bool_type = Box::new(Type::Primitive(primitives::Type::Bool));
    let generic_any = Rc::new(Generic {
        name: String::from("Value"),
        id: Uuid::new_v4(),
    });
    let generic_any_type = Box::new(Type::Generic(Rc::clone(&generic_any)));

    // TODO Can we somehow change it so constants is not passed explicitly?
    //  It seems that every use in other functions borrows this function as mutable, so that the
    //  reference can be accessed from there. Only one mutable borrow can exist, so two functions
    //  calling this one is impossible...?
    let add_function = |constants: &mut Box<HashMap<String, Rc<Variable>>>, interface: Rc<FunctionInterface>| -> Rc<FunctionInterface> {
        constants.insert(interface.name.clone(), create_function_variable(&interface));
        interface
    };

    // For now it's ok to assume the shape_returns types to be equal
    let add_binary_aa_x = |constants: &mut Box<HashMap<String, Rc<Variable>>>, name: &str, generic: &Rc<Generic>, return_type: &Box<Type>| -> Rc<FunctionInterface> {
        let generic_type = Box::new(Type::Generic(Rc::clone(generic)));

        add_function(constants, Rc::new(FunctionInterface {
            id: Uuid::new_v4(),
            name: String::from(name),
            parameters: create_same_parameters(
                &generic_type,
                vec!["lhs", "rhs"]
            ),
            return_type: Some(return_type.clone()),
            generics: vec![Rc::clone(generic)],
        }))
    };

    let add_binary_xx_x = |constants: &mut Box<HashMap<String, Rc<Variable>>>, name: &str, declared_type: &Box<Type>| -> Rc<FunctionInterface> {
        add_function(constants, Rc::new(FunctionInterface {
            id: Uuid::new_v4(),
            name: String::from(name),
            parameters: create_same_parameters(
                &declared_type,
                vec!["lhs", "rhs"]
            ),
            return_type: Some(declared_type.clone()),
            generics: vec![],
        }))
    };

    let primitive_metatypes = primitives::Type::iter()
        .map(|x| (x, Box::new(Type::MetaType(Box::new(Type::Primitive(x))))))
        .collect::<HashMap<primitives::Type, Box<Type>>>();

    for (primitive_type, metatype) in &primitive_metatypes {
        constants.insert(primitive_type.identifier_string(), Rc::new(Variable {
            id: Uuid::new_v4(),
            name: primitive_type.identifier_string(),
            type_declaration: metatype.clone(),
            mutability: Mutability::Immutable
        }));
    }

    let add_struct = |constants: &mut Box<HashMap<String, Rc<Variable>>>, name: &str| -> Rc<Struct> {
        let s = Rc::new(Struct {
            id: Uuid::new_v4(),
            name: String::from(name),
        });
        let s_type = Box::new(Type::MetaType(Box::new(Type::Struct(Rc::clone(&s)))));

        constants.insert(s.name.clone(), Rc::new(Variable {
            id: Uuid::new_v4(),
            name: s.name.clone(),
            type_declaration: s_type,
            mutability: Mutability::Immutable,
        }));

        s
    };

    Rc::new(TenLangBuiltins {
        operators: TenLangBuiltinOperators {
            and: add_binary_xx_x(&mut constants, "&&", &bool_type),
            or: add_binary_xx_x(&mut constants, "||", &bool_type),

            // These are n-ary in syntax but binary in implementation.
            equal_to: add_binary_aa_x(&mut constants, "==", &generic_any, &bool_type),
            not_equal_to: add_binary_aa_x(&mut constants, "!=", &generic_any, &bool_type),

            greater_than: add_binary_aa_x(&mut constants, ">", &generic_any, &bool_type),
            greater_than_or_equal_to: add_binary_aa_x(&mut constants, ">=", &generic_any, &bool_type),
            lesser_than: add_binary_aa_x(&mut constants, "<", &generic_any, &bool_type),
            lesser_than_or_equal_to: add_binary_aa_x(&mut constants, "<=", &generic_any, &bool_type),

            add: add_binary_aa_x(&mut constants, "+", &generic_any, &generic_any_type),
            subtract: add_binary_aa_x(&mut constants, "-", &generic_any, &generic_any_type),
            multiply: add_binary_aa_x(&mut constants, "*", &generic_any, &generic_any_type),
            divide: add_binary_aa_x(&mut constants, "/", &generic_any, &generic_any_type),
            exponentiate: add_binary_aa_x(&mut constants, "**", &generic_any, &generic_any_type),
            modulo: add_binary_aa_x(&mut constants, "%", &generic_any, &generic_any_type),

            // TODO These should be unary, and then renamed (when function overloading works)
            positive: add_binary_aa_x(&mut constants, "..+", &generic_any, &generic_any_type),
            negative: add_binary_aa_x(&mut constants, "..-", &generic_any, &generic_any_type),
            not: add_binary_aa_x(&mut constants, "!", &generic_any, &generic_any_type),
        },
        functions: TenLangBuiltinFunctions {
            print: add_function(&mut constants, Rc::new(FunctionInterface {
                id: Uuid::new_v4(),
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
