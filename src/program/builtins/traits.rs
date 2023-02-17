use uuid::Uuid;
use std::rc::Rc;
use std::collections::{HashMap, HashSet};
use crate::linker::scopes::Scope;
use crate::program::functions::{AbstractFunction, FunctionInterface, FunctionPointer};
use crate::program::primitives;
use crate::program::traits::Trait;
use crate::program::types::{TypeProto, TypeUnit};

#[allow(non_snake_case)]
pub struct Traits {
    pub all: HashSet<Rc<Trait>>,
    pub self_id: Uuid,

    pub Eq: Rc<Trait>,
    pub Eq_functions: EqFunctions<AbstractFunction>,

    pub Ord: Rc<Trait>,

    pub String: Rc<Trait>,

    pub ConstructableByIntLiteral: Rc<Trait>,
    pub parse_int_literal_function: Rc<AbstractFunction>,

    pub ConstructableByFloatLiteral: Rc<Trait>,
    pub parse_float_literal_function: Rc<AbstractFunction>,

    pub Number: Rc<Trait>,
    pub Number_functions: NumberFunctions<AbstractFunction>,

    pub Float: Rc<Trait>,
    pub Float_functions: FloatFunctions<AbstractFunction>,

    pub Int: Rc<Trait>,
}

pub struct EqFunctions<T> {
    pub equal_to: Rc<T>,
    pub not_equal_to: Rc<T>,
}

pub fn make_eq_functions<T>(type_: &Box<TypeProto>, f: fn(Rc<FunctionInterface>) -> Rc<T>) -> EqFunctions<T> {
    let bool_type = TypeProto::unit(TypeUnit::Primitive(primitives::Type::Bool));

    EqFunctions {
        equal_to: f(FunctionInterface::new_operator("is_equal", 2, type_, &bool_type)),
        not_equal_to: f(FunctionInterface::new_operator("is_not_equal", 2, type_, &bool_type)),
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

pub fn make_number_functions<T>(type_: &Box<TypeProto>, f: fn(Rc<FunctionInterface>) -> Rc<T>) -> NumberFunctions<T> {
    let bool_type = TypeProto::unit(TypeUnit::Primitive(primitives::Type::Bool));

    NumberFunctions {
        add: f(FunctionInterface::new_operator("add", 2, type_, type_)),
        subtract: f(FunctionInterface::new_operator("subtract", 2, type_, type_)),
        multiply: f(FunctionInterface::new_operator("multiply", 2, type_, type_)),
        divide: f(FunctionInterface::new_operator("divide", 2, type_, type_)),

        positive: f(FunctionInterface::new_operator("positive", 1, type_, type_)),
        negative: f(FunctionInterface::new_operator("negative", 1, type_, type_)),

        modulo: f(FunctionInterface::new_operator("modulo", 2, type_, type_)),

        greater_than: f(FunctionInterface::new_operator("is_greater", 2, type_, &bool_type)),
        greater_than_or_equal_to: f(FunctionInterface::new_operator("is_greater_or_equal", 2, type_, &bool_type)),
        lesser_than: f(FunctionInterface::new_operator("is_lesser", 2, type_, &bool_type)),
        lesser_than_or_equal_to: f(FunctionInterface::new_operator("is_lesser_or_equal", 2, type_, &bool_type)),
    }
}

pub fn make_trait(name: &str, self_id: &Uuid, fns: Vec<&Rc<AbstractFunction>>, parents: Vec<&Rc<Trait>>) -> Rc<Trait> {
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

pub fn make(constants: &mut Scope) -> Traits {
    let self_id = Uuid::new_v4();
    let self_type = TypeProto::unit(TypeUnit::Any(self_id));

    let eq_functions = make_eq_functions(&self_type, AbstractFunction::new);
    let eq_trait = make_trait("Eq", &self_id, vec![
        &eq_functions.equal_to,
        &eq_functions.not_equal_to,
    ], vec![]);
    constants.insert_trait(&eq_trait);

    let number_functions = make_number_functions(&self_type, AbstractFunction::new);

    let ord_trait = make_trait("Ord", &self_id, vec![
        &number_functions.greater_than,
        &number_functions.greater_than_or_equal_to,
        &number_functions.lesser_than,
        &number_functions.lesser_than_or_equal_to,
    ], vec![&eq_trait]);
    constants.insert_trait(&ord_trait);

    let number_trait = make_trait("Number", &self_id, vec![
        &number_functions.add,
        &number_functions.subtract,
        &number_functions.multiply,
        &number_functions.divide,
        &number_functions.positive,
        &number_functions.negative,
        &number_functions.modulo,
    ], vec![&ord_trait]);
    constants.insert_trait(&number_trait);


    let String = make_trait("String", &self_id, vec![], vec![]);
    constants.insert_trait(&String);


    let parse_int_literal_function = AbstractFunction::new(FunctionInterface::new_global(
        "parse_int_literal",
        [TypeProto::unit(TypeUnit::Struct(Rc::clone(&String)))].into_iter(),
        self_type.clone(),
    ));

    let ConstructableByIntLiteral = make_trait("ConstructableByIntLiteral", &self_id, vec![&parse_int_literal_function], vec![]);
    constants.insert_trait(&ConstructableByIntLiteral);


    let parse_float_literal_function = AbstractFunction::new(FunctionInterface::new_global(
        "parse_float_literal",
        [TypeProto::unit(TypeUnit::Struct(Rc::clone(&String)))].into_iter(),
        self_type.clone(),
    ));

    let ConstructableByFloatLiteral = make_trait("ConstructableByFloatLiteral", &self_id, vec![&parse_float_literal_function], vec![]);
    constants.insert_trait(&ConstructableByFloatLiteral);


    let float_functions = make_float_functions(&self_type, AbstractFunction::new);

    let float_trait = make_trait(
        "Float",
        &self_id,
        vec![&float_functions.exponent],
        vec![&number_trait, &ConstructableByFloatLiteral, &ConstructableByIntLiteral]
    );
    constants.insert_trait(&float_trait);

    let int_trait = make_trait(
        "Int",
        &self_id,
        vec![],
        vec![&number_trait, &ConstructableByIntLiteral]
    );
    constants.insert_trait(&int_trait);

    let traits = Traits {
        all: [&eq_trait, &ord_trait, &number_trait, &float_trait, &int_trait, &ConstructableByFloatLiteral, &ConstructableByIntLiteral, &String].map(Rc::clone).into_iter().collect(),
        self_id,

        Eq: eq_trait,
        Eq_functions: eq_functions,

        Ord: ord_trait,

        String,

        ConstructableByIntLiteral,
        parse_int_literal_function,
        ConstructableByFloatLiteral,
        parse_float_literal_function,

        Number: number_trait,
        Number_functions: number_functions,

        Float: float_trait,
        Float_functions: float_functions,

        Int: int_trait,
    };
    traits
}
