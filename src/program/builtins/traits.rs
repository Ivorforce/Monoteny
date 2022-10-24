use uuid::Uuid;
use std::rc::Rc;
use std::collections::HashSet;
use crate::linker::scopes::Scope;
use crate::program::functions::FunctionPointer;
use crate::program::primitives;
use crate::program::traits::Trait;
use crate::program::types::{TypeProto, TypeUnit};

#[allow(non_snake_case)]
pub struct Traits {
    pub all: HashSet<Rc<Trait>>,

    pub Eq: Rc<Trait>,
    pub Eq_functions: EqFunctions,

    pub Ord: Rc<Trait>,

    pub Number: Rc<Trait>,
    pub Number_functions: NumberFunctions,

    pub Float: Rc<Trait>,
    pub Float_functions: FloatFunctions,

    pub Int: Rc<Trait>,
}

pub struct EqFunctions {
    pub equal_to: Rc<FunctionPointer>,
    pub not_equal_to: Rc<FunctionPointer>,
}

pub fn make_eq_functions(type_: &Box<TypeProto>) -> EqFunctions {
    let bool_type = TypeProto::unit(TypeUnit::Primitive(primitives::Type::Bool));

    EqFunctions {
        equal_to: FunctionPointer::make_operator("is_equal", 2, type_, &bool_type),
        not_equal_to: FunctionPointer::make_operator("is_not_equal", 2, type_, &bool_type),
    }
}

pub struct NumberFunctions {
    // Ord
    pub greater_than: Rc<FunctionPointer>,
    pub greater_than_or_equal_to: Rc<FunctionPointer>,
    pub lesser_than: Rc<FunctionPointer>,
    pub lesser_than_or_equal_to: Rc<FunctionPointer>,

    // Number
    pub add: Rc<FunctionPointer>,
    pub subtract: Rc<FunctionPointer>,
    pub multiply: Rc<FunctionPointer>,
    pub divide: Rc<FunctionPointer>,

    pub modulo: Rc<FunctionPointer>,

    pub positive: Rc<FunctionPointer>,
    pub negative: Rc<FunctionPointer>,
}

pub fn make_number_functions(type_: &Box<TypeProto>) -> NumberFunctions {
    let bool_type = TypeProto::unit(TypeUnit::Primitive(primitives::Type::Bool));

    NumberFunctions {
        add: FunctionPointer::make_operator("add", 2, type_, type_),
        subtract: FunctionPointer::make_operator("subtract", 2, type_, type_),
        multiply: FunctionPointer::make_operator("multiply", 2, type_, type_),
        divide: FunctionPointer::make_operator("divide", 2, type_, type_),

        positive: FunctionPointer::make_operator("positive", 1, type_, type_),
        negative: FunctionPointer::make_operator("negative", 1, type_, type_),

        modulo: FunctionPointer::make_operator("modulo", 2, type_, type_),

        greater_than: FunctionPointer::make_operator("is_greater", 2, type_, &bool_type),
        greater_than_or_equal_to: FunctionPointer::make_operator("is_greater_or_equal", 2, type_, &bool_type),
        lesser_than: FunctionPointer::make_operator("is_lesser", 2, type_, &bool_type),
        lesser_than_or_equal_to: FunctionPointer::make_operator("is_lesser_or_equal", 2, type_, &bool_type),
    }
}

pub struct FloatFunctions {
    pub exponent: Rc<FunctionPointer>,
    pub logarithm: Rc<FunctionPointer>,
}

pub fn make_float_functions(type_: &Box<TypeProto>) -> FloatFunctions {
    FloatFunctions {
        exponent: FunctionPointer::make_operator("exponent", 2, type_, type_),
        logarithm: FunctionPointer::make_operator("logarithm", 2, type_, type_),
    }
}

pub fn make(constants: &mut Scope) -> Traits {
    let generic_id = Uuid::new_v4();
    let generic_type = TypeProto::unit(TypeUnit::Any(generic_id));

    let make_trait = |name: &str, generic_id: &Uuid, fns: Vec<&Rc<FunctionPointer>>, parents: Vec<Rc<Trait>>| -> Rc<Trait> {
        let generic_type = TypeProto::unit(TypeUnit::Any(*generic_id));

        let mut t = Trait {
            id: Uuid::new_v4(),
            name: String::from(name),
            parameters: vec![*generic_id],
            abstract_functions: fns.into_iter().map(Rc::clone).collect(),
            requirements: HashSet::new(),
        };

        for parent in parents {
            t.requirements.insert(Trait::require(&parent, vec![generic_type.clone()]));
        }

        return Rc::new(t)
    };

    let eq_functions = make_eq_functions(&generic_type);
    let eq_trait = make_trait("Eq", &generic_id, vec![
        &eq_functions.equal_to,
        &eq_functions.not_equal_to,
    ], vec![]);
    constants.insert_trait(&eq_trait);

    let number_functions = make_number_functions(&generic_type);

    let ord_trait = make_trait("Ord", &generic_id, vec![
        &number_functions.greater_than,
        &number_functions.greater_than_or_equal_to,
        &number_functions.lesser_than,
        &number_functions.lesser_than_or_equal_to,
    ], vec![Rc::clone(&eq_trait)]);
    constants.insert_trait(&ord_trait);

    let number_trait = make_trait("Number", &generic_id, vec![
        &number_functions.add,
        &number_functions.subtract,
        &number_functions.multiply,
        &number_functions.divide,
        &number_functions.positive,
        &number_functions.negative,
        &number_functions.modulo,
    ], vec![Rc::clone(&ord_trait)]);
    constants.insert_trait(&number_trait);

    let float_functions = make_float_functions(&generic_type);

    let float_trait = make_trait("Float", &generic_id, vec![
        &float_functions.exponent,
    ], vec![Rc::clone(&number_trait)]);
    constants.insert_trait(&float_trait);

    let int_trait = make_trait("Int", &generic_id, vec![], vec![Rc::clone(&number_trait)]);
    constants.insert_trait(&int_trait);

    let traits = Traits {
        all: [&eq_trait, &ord_trait, &number_trait, &float_trait, &int_trait].map(Rc::clone).into_iter().collect(),

        Eq: eq_trait,
        Eq_functions: eq_functions,

        Ord: ord_trait,

        Number: number_trait,
        Number_functions: number_functions,

        Float: float_trait,
        Float_functions: float_functions,

        Int: int_trait,
    };
    traits
}
