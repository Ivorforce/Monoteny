use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;
use crate::abstract_syntax::Mutability;
use crate::linker::computation_tree::*;
use crate::linker::scopes::Scope;

pub struct TenLangBuiltins {
    pub operators: TenLangBuiltinOperators,
    pub functions: TenLangBuiltinFunctions,
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

pub fn create_builtins() -> Rc<TenLangBuiltins> {
    let mut constants: Box<HashMap<String, Rc<Variable>>> = Box::new(HashMap::new());

    let mut add_function = |name: &str, parameters: Vec<Box<Parameter>>, generics: Vec<Rc<Generic>>, return_type: Option<Box<Type>>| -> Rc<FunctionInterface> {
        let interface = Rc::new(FunctionInterface {
            id: Uuid::new_v4(),
            name: String::from(name),
            return_type,
            parameters,
            generics,
        });

        let var = Rc::new(Variable {
            id: Uuid::new_v4(),
            name: String::from(name),
            type_declaration: Box::new(Type::Function(interface.clone())),
            mutability: Mutability::Immutable,
        });
        constants.insert(var.name.clone(), Rc::clone(&var));

        return interface;
    };

    // For now it's ok to assume the 3 types to be equal
    let mut add_binary_operator = |name: &str| -> Rc<FunctionInterface> {
        let generic = Rc::new(Generic {
            name: String::from("Value"),
            id: Uuid::new_v4(),
        });
        let generic_type = Box::new(Type::Generic(Rc::clone(&generic)));

        let parameters: Vec<Box<Parameter>> = vec!["lhs", "rhs"].iter().enumerate()
            .map(|(idx, name)| Box::new(Parameter {
                external_key: ParameterKey::Int(idx as i32),
                variable: Rc::new(Variable {
                    id: Uuid::new_v4(),
                    name: String::from(*name),
                    type_declaration: generic_type.clone(),
                    mutability: Mutability::Immutable,
                }),
            })).collect();

        return add_function(name, parameters, vec![generic], Some(generic_type));
    };

    Rc::new(TenLangBuiltins {
        operators: TenLangBuiltinOperators {
            and: add_binary_operator("&&"),
            or: add_binary_operator("||"),

            // These are n-ary in syntax but binary in implementation.
            equal_to: add_binary_operator("=="),
            not_equal_to: add_binary_operator("!="),

            greater_than: add_binary_operator(">"),
            greater_than_or_equal_to: add_binary_operator(">="),
            lesser_than: add_binary_operator("<"),
            lesser_than_or_equal_to: add_binary_operator("<="),

            add: add_binary_operator("+"),
            subtract: add_binary_operator("-"),
            multiply: add_binary_operator("*"),
            divide: add_binary_operator("/"),
            exponentiate: add_binary_operator("**"),
            modulo: add_binary_operator("%"),

            // TODO These should be unary
            positive: add_binary_operator("+"),
            negative: add_binary_operator("-"),
            not: add_binary_operator("!"),
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
                        }),
                    })
                ],
                vec![],
                None,
            )
        },
        global_constants: constants,
    })
}
