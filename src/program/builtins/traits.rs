use std::collections::HashMap;
use std::rc::Rc;
use crate::program::functions::{FunctionInterface, FunctionPointer};
use crate::program::module::Module;
use crate::program::primitives;
use crate::program::traits::Trait;
use crate::program::types::{TypeProto, TypeUnit};


pub struct Traits {
    pub Eq: Rc<Trait>,
    pub Eq_functions: EqFunctions,

    pub Ord: Rc<Trait>,
    pub Ord_functions: OrdFunctions,

    pub String: Rc<Trait>,

    pub ConstructableByIntLiteral: Rc<Trait>,
    pub parse_int_literal_function: Rc<FunctionPointer>,

    pub ConstructableByFloatLiteral: Rc<Trait>,
    pub parse_float_literal_function: Rc<FunctionPointer>,

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

pub fn make_eq_functions(type_: &Box<TypeProto>, bool_type: &Box<TypeProto>) -> EqFunctions {
    EqFunctions {
        equal_to: FunctionPointer::new_global(
            "is_equal",
            FunctionInterface::new_operator(2, type_, bool_type)
        ),
        not_equal_to: FunctionPointer::new_global(
            "is_not_equal",
            FunctionInterface::new_operator(2, type_, bool_type)
        ),
    }
}

pub struct OrdFunctions {
    pub greater_than: Rc<FunctionPointer>,
    pub greater_than_or_equal_to: Rc<FunctionPointer>,
    pub lesser_than: Rc<FunctionPointer>,
    pub lesser_than_or_equal_to: Rc<FunctionPointer>,
}

pub fn make_ord_functions(type_: &Box<TypeProto>, bool_type: &Box<TypeProto>) -> OrdFunctions {
    OrdFunctions {
        greater_than: FunctionPointer::new_global(
            "is_greater",
            FunctionInterface::new_operator(2, type_, bool_type)
        ),
        greater_than_or_equal_to: FunctionPointer::new_global(
            "is_greater_or_equal",
            FunctionInterface::new_operator(2, type_, bool_type)
        ),
        lesser_than: FunctionPointer::new_global(
            "is_lesser",
            FunctionInterface::new_operator(2, type_, bool_type)
        ),
        lesser_than_or_equal_to: FunctionPointer::new_global(
            "is_greater_or_equal",
            FunctionInterface::new_operator(2, type_, bool_type)
        ),
    }
}

pub struct NumberFunctions {
    pub add: Rc<FunctionPointer>,
    pub subtract: Rc<FunctionPointer>,
    pub multiply: Rc<FunctionPointer>,
    pub divide: Rc<FunctionPointer>,

    pub modulo: Rc<FunctionPointer>,

    pub negative: Rc<FunctionPointer>,
}

pub fn make_number_functions(type_: &Box<TypeProto>, bool_type: &Box<TypeProto>) -> NumberFunctions {
    NumberFunctions {
        add: FunctionPointer::new_global(
            "add",
            FunctionInterface::new_operator(2, type_, type_)
        ),
        subtract: FunctionPointer::new_global(
            "subtract",
            FunctionInterface::new_operator(2, type_, type_)
        ),
        multiply: FunctionPointer::new_global(
            "multiply",
            FunctionInterface::new_operator(2, type_, type_)
        ),
        divide: FunctionPointer::new_global(
            "divide",
            FunctionInterface::new_operator(2, type_, type_)
        ),

        negative: FunctionPointer::new_global(
            "negative",
            FunctionInterface::new_operator(1, type_, type_)
        ),

        modulo: FunctionPointer::new_global(
            "modulo",
            FunctionInterface::new_operator(2, type_, type_)
        ),
    }
}

pub struct FloatFunctions {
    pub exponent: Rc<FunctionPointer>,
    pub logarithm: Rc<FunctionPointer>,
}

pub fn make_float_functions(type_: &Box<TypeProto>) -> FloatFunctions {
    FloatFunctions {
        exponent: FunctionPointer::new_global(
            "exponent",
            FunctionInterface::new_operator(2, type_, type_)
        ),
        logarithm: FunctionPointer::new_global(
            "log",
            FunctionInterface::new_operator(1, type_, type_)
        ),
    }
}

pub fn create(module: &mut Module, primitive_traits: &HashMap<primitives::Type, Rc<Trait>>) -> Traits {
    let bool_type = TypeProto::simple_struct(&primitive_traits[&primitives::Type::Bool]);

    let mut Eq = Trait::new("Eq".into());
    let eq_functions = make_eq_functions(&Eq.create_generic_type(&"self".into()), &bool_type);
    Eq.insert_functions([
        &eq_functions.equal_to,
        &eq_functions.not_equal_to,
    ].into_iter());
    let Eq = Rc::new(Eq);
    module.add_trait(&Eq);

    let mut Ord = Trait::new("Ord".into());
    let ord_functions = make_ord_functions(&Ord.create_generic_type(&"self".into()), &bool_type);
    Ord.insert_functions([
        &ord_functions.greater_than,
        &ord_functions.greater_than_or_equal_to,
        &ord_functions.lesser_than,
        &ord_functions.lesser_than_or_equal_to,
    ].into_iter());
    let Ord = Rc::new(Ord);
    module.add_trait(&Ord);
    module.trait_conformance.add_simple_parent_requirement(&Ord, &Eq);

    let mut Number = Trait::new("Number".into());
    let number_functions = make_number_functions(&Number.create_generic_type(&"self".into()), &bool_type);
    Number.insert_functions([
        &number_functions.add,
        &number_functions.subtract,
        &number_functions.multiply,
        &number_functions.divide,
        &number_functions.negative,
        &number_functions.modulo,
    ].into_iter());
    let Number = Rc::new(Number);
    module.add_trait(&Number);
    module.trait_conformance.add_simple_parent_requirement(&Number, &Ord);

    let mut String = Trait::new("String".into());
    let String = Rc::new(String);
    module.add_trait(&String);


    let mut ConstructableByIntLiteral = Trait::new("ConstructableByIntLiteral".into());
    let parse_int_literal_function = FunctionPointer::new_global(
        "parse_int_literal",
        FunctionInterface::new_simple(
            [TypeProto::unit(TypeUnit::Struct(Rc::clone(&String)))].into_iter(),
            ConstructableByIntLiteral.create_generic_type(&"self".into()),
        )
    );
    ConstructableByIntLiteral.insert_functions([
        &parse_int_literal_function
    ].into_iter());
    let ConstructableByIntLiteral = Rc::new(ConstructableByIntLiteral);
    module.add_trait(&ConstructableByIntLiteral);


    let mut ConstructableByFloatLiteral = Trait::new("ConstructableByFloatLiteral".into());
    let parse_float_literal_function = FunctionPointer::new_global(
        "parse_float_literal",
        FunctionInterface::new_simple(
            [TypeProto::unit(TypeUnit::Struct(Rc::clone(&String)))].into_iter(),
            ConstructableByFloatLiteral.create_generic_type(&"self".into())
        ),
    );
    ConstructableByFloatLiteral.insert_functions([
        &parse_float_literal_function
    ].into_iter());
    let ConstructableByFloatLiteral = Rc::new(ConstructableByFloatLiteral);
    module.add_trait(&ConstructableByFloatLiteral);


    let mut Float = Trait::new("Float".into());
    let float_functions = make_float_functions(&Float.create_generic_type(&"self".into()));
    Float.insert_functions([
        &float_functions.exponent,
        &float_functions.logarithm
    ].into_iter());
    let Float = Rc::new(Float);
    module.add_trait(&Float);
    module.trait_conformance.add_simple_parent_requirement(&Float, &Number);
    module.trait_conformance.add_simple_parent_requirement(&Float, &ConstructableByFloatLiteral);
    module.trait_conformance.add_simple_parent_requirement(&Float, &ConstructableByIntLiteral);

    let mut Int = Trait::new("Int".into());
    let Int = Rc::new(Int);
    module.add_trait(&Int);
    module.trait_conformance.add_simple_parent_requirement(&Int, &Number);
    module.trait_conformance.add_simple_parent_requirement(&Int, &ConstructableByIntLiteral);

    Traits {
        Eq,
        Eq_functions: eq_functions,

        Ord,
        Ord_functions: ord_functions,

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
