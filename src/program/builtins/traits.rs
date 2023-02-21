use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use uuid::Uuid;
use crate::program::functions::{Function, FunctionInterface};
use crate::program::module::Module;
use crate::program::{builtins, primitives};
use crate::program::traits::Trait;
use crate::program::types::{TypeProto, TypeUnit};


pub struct Traits {
    pub Eq: Rc<Trait>,
    pub Eq_functions: EqFunctions<Function>,

    pub Ord: Rc<Trait>,

    pub String: Rc<Trait>,

    pub ConstructableByIntLiteral: Rc<Trait>,
    pub parse_int_literal_function: Rc<Function>,

    pub ConstructableByFloatLiteral: Rc<Trait>,
    pub parse_float_literal_function: Rc<Function>,

    pub Number: Rc<Trait>,
    pub Number_functions: NumberFunctions<Function>,

    pub Float: Rc<Trait>,
    pub Float_functions: FloatFunctions<Function>,

    pub Int: Rc<Trait>,
}


pub struct EqFunctions<T> {
    pub equal_to: Rc<T>,
    pub not_equal_to: Rc<T>,
}

pub fn make_eq_functions<T>(type_: &Box<TypeProto>, bool_type: &Box<TypeProto>, f: fn(Rc<FunctionInterface>) -> Rc<T>) -> EqFunctions<T> {
    EqFunctions {
        equal_to: f(FunctionInterface::new_operator("is_equal", 2, type_, bool_type)),
        not_equal_to: f(FunctionInterface::new_operator("is_not_equal", 2, type_, bool_type)),
    }
}

pub struct NumberFunctions<T> {
    // Ord
    pub greater_than: Rc<T>,
    pub greater_than_or_equal_to: Rc<T>,
    pub lesser_than: Rc<T>,
    pub lesser_than_or_equal_to: Rc<T>,

    // Number
    pub add: Rc<T>,
    pub subtract: Rc<T>,
    pub multiply: Rc<T>,
    pub divide: Rc<T>,

    pub modulo: Rc<T>,

    pub positive: Rc<T>,
    pub negative: Rc<T>,
}

pub fn make_number_functions<T>(type_: &Box<TypeProto>, bool_type: &Box<TypeProto>, f: fn(Rc<FunctionInterface>) -> Rc<T>) -> NumberFunctions<T> {
    NumberFunctions {
        add: f(FunctionInterface::new_operator("add", 2, type_, type_)),
        subtract: f(FunctionInterface::new_operator("subtract", 2, type_, type_)),
        multiply: f(FunctionInterface::new_operator("multiply", 2, type_, type_)),
        divide: f(FunctionInterface::new_operator("divide", 2, type_, type_)),

        positive: f(FunctionInterface::new_operator("positive", 1, type_, type_)),
        negative: f(FunctionInterface::new_operator("negative", 1, type_, type_)),

        modulo: f(FunctionInterface::new_operator("modulo", 2, type_, type_)),

        greater_than: f(FunctionInterface::new_operator("is_greater", 2, type_, bool_type)),
        greater_than_or_equal_to: f(FunctionInterface::new_operator("is_greater_or_equal", 2, type_, bool_type)),
        lesser_than: f(FunctionInterface::new_operator("is_lesser", 2, type_, bool_type)),
        lesser_than_or_equal_to: f(FunctionInterface::new_operator("is_lesser_or_equal", 2, type_, bool_type)),
    }
}

pub fn make_trait(name: &str, self_id: &Uuid, fns: Vec<&Rc<Function>>, parents: Vec<&Rc<Trait>>) -> Rc<Trait> {
    let self_type = TypeProto::unit(TypeUnit::Any(*self_id));

    let mut t = Trait {
        id: Uuid::new_v4(),
        name: String::from(name),
        generics: HashSet::from([*self_id]),
        abstract_functions: fns.into_iter().map(Rc::clone).collect(),
        requirements: HashSet::new(),
    };

    for parent in parents {
        assert_eq!(parent.generics.len(), 1);
        t.requirements.insert(
            Trait::require(parent, HashMap::from([(*parent.generics.iter().next().unwrap(), self_type.clone())]))
        );
    }

    return Rc::new(t)
}

pub struct FloatFunctions<T> {
    pub exponent: Rc<T>,
    pub logarithm: Rc<T>,
}

pub fn make_float_functions<T>(type_: &Box<TypeProto>, f: fn(Rc<FunctionInterface>) -> Rc<T>) -> FloatFunctions<T> {
    FloatFunctions {
        exponent: f(FunctionInterface::new_operator("exponent", 2, type_, type_)),
        logarithm: f(FunctionInterface::new_operator("logarithm", 2, type_, type_)),
    }
}

pub fn create(module: &mut Module, primitive_traits: &HashMap<primitives::Type, Rc<Trait>>) -> Traits {
    let self_id = Uuid::new_v4();
    let self_type = TypeProto::unit(TypeUnit::Any(self_id));
    let bool_type = TypeProto::simple_struct(&primitive_traits[&primitives::Type::Bool]);

    let eq_functions = make_eq_functions(&self_type, &bool_type, Function::new);
    let Eq = make_trait("Eq", &self_id, vec![
        &eq_functions.equal_to,
        &eq_functions.not_equal_to,
    ], vec![]);
    module.add_trait(&Eq);

    let number_functions = make_number_functions(&self_type, &bool_type, Function::new);

    let Ord = make_trait("Ord", &self_id, vec![
        &number_functions.greater_than,
        &number_functions.greater_than_or_equal_to,
        &number_functions.lesser_than,
        &number_functions.lesser_than_or_equal_to,
    ], vec![&Eq]);
    module.add_trait(&Ord);

    let Number = make_trait("Number", &self_id, vec![
        &number_functions.add,
        &number_functions.subtract,
        &number_functions.multiply,
        &number_functions.divide,
        &number_functions.positive,
        &number_functions.negative,
        &number_functions.modulo,
    ], vec![&Ord]);
    module.add_trait(&Number);


    let String = make_trait("String", &self_id, vec![], vec![]);
    module.add_trait(&String);


    let parse_int_literal_function = Function::new(FunctionInterface::new_global(
        "parse_int_literal",
        [TypeProto::unit(TypeUnit::Struct(Rc::clone(&String)))].into_iter(),
        self_type.clone(),
    ));

    let ConstructableByIntLiteral = make_trait("ConstructableByIntLiteral", &self_id, vec![&parse_int_literal_function], vec![]);
    module.add_trait(&ConstructableByIntLiteral);


    let parse_float_literal_function = Function::new(FunctionInterface::new_global(
        "parse_float_literal",
        [TypeProto::unit(TypeUnit::Struct(Rc::clone(&String)))].into_iter(),
        self_type.clone(),
    ));

    let ConstructableByFloatLiteral = make_trait("ConstructableByFloatLiteral", &self_id, vec![&parse_float_literal_function], vec![]);
    module.add_trait(&ConstructableByFloatLiteral);


    let float_functions = make_float_functions(&self_type, Function::new);

    let Float = make_trait(
        "Float",
        &self_id,
        vec![&float_functions.exponent],
        vec![&Number, &ConstructableByFloatLiteral, &ConstructableByIntLiteral]
    );
    module.add_trait(&Float);

    let Int = make_trait(
        "Int",
        &self_id,
        vec![],
        vec![&Number, &ConstructableByIntLiteral]
    );
    module.add_trait(&Int);

    Traits {
        Eq,
        Eq_functions: eq_functions,

        Ord,

        String,

        ConstructableByIntLiteral,
        parse_int_literal_function,
        ConstructableByFloatLiteral,
        parse_float_literal_function,

        Number,
        Number_functions: number_functions,

        Float,
        Float_functions: float_functions,

        Int,
    }
}
