use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use uuid::Uuid;
use crate::program::functions::{Function, FunctionCallType, FunctionForm, FunctionInterface, FunctionPointer};
use crate::program::module::Module;
use crate::program::{builtins, primitives};
use crate::program::traits::Trait;
use crate::program::types::{TypeProto, TypeUnit};


pub struct Traits {
    pub Eq: Rc<Trait>,
    pub Eq_functions: EqFunctions,

    pub Ord: Rc<Trait>,

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

        positive: FunctionPointer::new_global(
            "positive",
            FunctionInterface::new_operator(1, type_, type_)
        ),
        negative: FunctionPointer::new_global(
            "negative",
            FunctionInterface::new_operator(1, type_, type_)
        ),

        modulo: FunctionPointer::new_global(
            "modulo",
            FunctionInterface::new_operator(2, type_, type_)
        ),

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

pub fn make_trait(name: &str, self_id: &Uuid, fns: Vec<&Rc<FunctionPointer>>, parents: Vec<&Rc<Trait>>) -> Rc<Trait> {
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
            "logarithm",
            FunctionInterface::new_operator(2, type_, type_)
        ),
    }
}

pub fn create(module: &mut Module, primitive_traits: &HashMap<primitives::Type, Rc<Trait>>) -> Traits {
    let self_id = Uuid::new_v4();
    let self_type = TypeProto::unit(TypeUnit::Any(self_id));
    let bool_type = TypeProto::simple_struct(&primitive_traits[&primitives::Type::Bool]);

    let eq_functions = make_eq_functions(&self_type, &bool_type);
    let Eq = make_trait("Eq", &self_id, vec![
        &eq_functions.equal_to,
        &eq_functions.not_equal_to,
    ], vec![]);
    module.add_trait(&Eq);

    let number_functions = make_number_functions(&self_type, &bool_type);

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


    let parse_int_literal_function = FunctionPointer::new_global(
        "parse_int_literal",
        FunctionInterface::new_simple(
            [TypeProto::unit(TypeUnit::Struct(Rc::clone(&String)))].into_iter(),
            self_type.clone(),
        )
    );

    let ConstructableByIntLiteral = make_trait("ConstructableByIntLiteral", &self_id, vec![&parse_int_literal_function], vec![]);
    module.add_trait(&ConstructableByIntLiteral);


    let parse_float_literal_function = FunctionPointer::new_global(
        "parse_float_literal",
        FunctionInterface::new_simple(
            [TypeProto::unit(TypeUnit::Struct(Rc::clone(&String)))].into_iter(),
            self_type.clone(),
        )
    );

    let ConstructableByFloatLiteral = make_trait("ConstructableByFloatLiteral", &self_id, vec![&parse_float_literal_function], vec![]);
    module.add_trait(&ConstructableByFloatLiteral);


    let float_functions = make_float_functions(&self_type);

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
