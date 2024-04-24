use std::rc::Rc;

use crate::interpreter::runtime::Runtime;
use crate::resolver::referencible;
use crate::program::function_object::{FunctionCallExplicity, FunctionRepresentation, FunctionTargetType};
use crate::program::functions::{FunctionHead, FunctionInterface};
use crate::program::module::Module;
use crate::program::primitives;
use crate::program::traits::Trait;
use crate::program::types::TypeProto;

pub struct FunctionPointer {
    pub target: Rc<FunctionHead>,
    pub representation: FunctionRepresentation,
}

impl FunctionPointer {
    pub fn new_global_function(name: &str, interface: Rc<FunctionInterface>) -> Rc<FunctionPointer> {
        Rc::new(FunctionPointer {
            target: FunctionHead::new_static(interface),
            representation: FunctionRepresentation {
                name: name.to_string(),
                target_type: FunctionTargetType::Global,
                call_explicity: FunctionCallExplicity::Explicit,
            },
        })
    }

    pub fn new_member_function(name: &str, interface: Rc<FunctionInterface>) -> Rc<FunctionPointer> {
        Rc::new(FunctionPointer {
            target: FunctionHead::new_static(interface),
            representation: FunctionRepresentation {
                name: name.to_string(),
                target_type: FunctionTargetType::Member,
                call_explicity: FunctionCallExplicity::Explicit,
            },
        })
    }

    pub fn new_global_implicit(name: &str, interface: Rc<FunctionInterface>) -> Rc<FunctionPointer> {
        Rc::new(FunctionPointer {
            target: FunctionHead::new_static(interface),
            representation: FunctionRepresentation {
                name: name.to_string(),
                target_type: FunctionTargetType::Global,
                call_explicity: FunctionCallExplicity::Implicit,
            },
        })
    }
}

pub fn insert_functions<'a, I>(module: &mut Trait, functions: I) where I: Iterator<Item=&'a Rc<FunctionPointer>> {
    for ptr in functions {
        module.insert_function(Rc::clone(&ptr.target), ptr.representation.clone())
    }
}

#[allow(non_snake_case)]
#[derive(Clone)]
pub struct Traits {
    /// Supertype of all function objects.
    /// No requirements yet (will require call_as_function to return self!).
    pub Function: Rc<Trait>,

    pub Eq: Rc<Trait>,
    pub Eq_functions: EqFunctions,

    pub Ord: Rc<Trait>,
    pub Ord_functions: OrdFunctions,

    pub String: Rc<Trait>,
    pub ToString: Rc<Trait>,
    pub to_string_function: Rc<FunctionPointer>,

    pub ConstructableByIntLiteral: Rc<Trait>,
    pub parse_int_literal_function: Rc<FunctionPointer>,

    pub ConstructableByRealLiteral: Rc<Trait>,
    pub parse_real_literal_function: Rc<FunctionPointer>,

    pub Number: Rc<Trait>,
    pub Number_functions: NumberFunctions,

    pub Real: Rc<Trait>,
    pub Real_functions: RealFunctions,

    pub Int: Rc<Trait>,
    pub Natural: Rc<Trait>,
}

#[derive(Clone)]
pub struct EqFunctions {
    pub equal_to: Rc<FunctionPointer>,
    pub not_equal_to: Rc<FunctionPointer>,
}

pub fn make_eq_functions(type_: &Rc<TypeProto>, bool_type: &Rc<TypeProto>) -> EqFunctions {
    EqFunctions {
        equal_to: FunctionPointer::new_global_function(
            "is_equal",
            FunctionInterface::new_operator(2, type_, bool_type)
        ),
        not_equal_to: FunctionPointer::new_global_function(
            "is_not_equal",
            FunctionInterface::new_operator(2, type_, bool_type)
        ),
    }
}

#[derive(Clone)]
pub struct OrdFunctions {
    pub greater_than: Rc<FunctionPointer>,
    pub greater_than_or_equal_to: Rc<FunctionPointer>,
    pub lesser_than: Rc<FunctionPointer>,
    pub lesser_than_or_equal_to: Rc<FunctionPointer>,
}

pub fn make_ord_functions(type_: &Rc<TypeProto>, bool_type: &Rc<TypeProto>) -> OrdFunctions {
    OrdFunctions {
        greater_than: FunctionPointer::new_global_function(
            "is_greater",
            FunctionInterface::new_operator(2, type_, bool_type)
        ),
        greater_than_or_equal_to: FunctionPointer::new_global_function(
            "is_greater_or_equal",
            FunctionInterface::new_operator(2, type_, bool_type)
        ),
        lesser_than: FunctionPointer::new_global_function(
            "is_lesser",
            FunctionInterface::new_operator(2, type_, bool_type)
        ),
        lesser_than_or_equal_to: FunctionPointer::new_global_function(
            "is_lesser_or_equal",
            FunctionInterface::new_operator(2, type_, bool_type)
        ),
    }
}

#[derive(Clone)]
pub struct NumberFunctions {
    pub add: Rc<FunctionPointer>,
    pub subtract: Rc<FunctionPointer>,
    pub multiply: Rc<FunctionPointer>,
    pub divide: Rc<FunctionPointer>,

    pub modulo: Rc<FunctionPointer>,

    /// You may argue that unsigned numbers should not need to support negative.
    /// However, all unsigned numbers have rollover. That means that e.g. -1 = MAX, and
    ///  it's generally a perfectly valid operation.
    pub negative: Rc<FunctionPointer>,
}

pub fn make_number_functions(type_: &Rc<TypeProto>) -> NumberFunctions {
    NumberFunctions {
        add: FunctionPointer::new_global_function(
            "add",
            FunctionInterface::new_operator(2, type_, type_)
        ),
        subtract: FunctionPointer::new_global_function(
            "subtract",
            FunctionInterface::new_operator(2, type_, type_)
        ),
        multiply: FunctionPointer::new_global_function(
            "multiply",
            FunctionInterface::new_operator(2, type_, type_)
        ),
        divide: FunctionPointer::new_global_function(
            "divide",
            FunctionInterface::new_operator(2, type_, type_)
        ),

        negative: FunctionPointer::new_global_function(
            "negative",
            FunctionInterface::new_operator(1, type_, type_)
        ),

        modulo: FunctionPointer::new_global_function(
            "modulo",
            FunctionInterface::new_operator(2, type_, type_)
        ),
    }
}

#[derive(Clone)]
pub struct RealFunctions {
    pub pow: Rc<FunctionPointer>,
    pub log: Rc<FunctionPointer>,
}

pub fn make_real_functions(type_: &Rc<TypeProto>) -> RealFunctions {
    RealFunctions {
        pow: FunctionPointer::new_global_function(
            "pow",
            FunctionInterface::new_operator(2, type_, type_)
        ),
        log: FunctionPointer::new_global_function(
            "log",
            FunctionInterface::new_operator(1, type_, type_)
        ),
    }
}

#[allow(non_snake_case)]
pub fn make_to_string_function(type_: &Trait, String: &Rc<Trait>) -> Rc<FunctionPointer> {
    FunctionPointer::new_member_function(
        "to_string",
        FunctionInterface::new_member(
            type_.create_generic_type("Self"),
            [].into_iter(),
            TypeProto::unit_struct(&String)
        )
    )
}

#[allow(non_snake_case)]
pub fn create(runtime: &mut Runtime, module: &mut Module) -> Traits {
    let primitive_traits = runtime.primitives.as_ref().unwrap();
    let bool_type = TypeProto::unit_struct(&primitive_traits[&primitives::Type::Bool]);

    let mut Function = Trait::new_with_self("Function");
    let Function = Rc::new(Function);
    referencible::add_trait(runtime, module, None, &Function).unwrap();

    let mut Eq = Trait::new_with_self("Eq");
    let eq_functions = make_eq_functions(&Eq.create_generic_type("Self"), &bool_type);
    insert_functions(&mut Eq, [
        &eq_functions.equal_to,
        &eq_functions.not_equal_to,
    ].into_iter());
    let Eq = Rc::new(Eq);
    referencible::add_trait(runtime, module, None, &Eq).unwrap();

    let mut Ord = Trait::new_with_self("Ord");
    let ord_functions = make_ord_functions(&Ord.create_generic_type("Self"), &bool_type);
    insert_functions(&mut Ord, [
        &ord_functions.greater_than,
        &ord_functions.greater_than_or_equal_to,
        &ord_functions.lesser_than,
        &ord_functions.lesser_than_or_equal_to,
    ].into_iter());
    Ord.add_simple_parent_requirement(&Eq);
    let Ord = Rc::new(Ord);
    referencible::add_trait(runtime, module, None, &Ord).unwrap();

    let mut Number = Trait::new_with_self("Number");
    let number_functions = make_number_functions(&Number.create_generic_type("Self"));
    insert_functions(&mut Number, [
        &number_functions.add,
        &number_functions.subtract,
        &number_functions.multiply,
        &number_functions.divide,
        &number_functions.negative,
        &number_functions.modulo,
    ].into_iter());
    Number.add_simple_parent_requirement(&Ord);
    let Number = Rc::new(Number);
    referencible::add_trait(runtime, module, None, &Number).unwrap();

    let mut String = Trait::new_with_self("String");
    let String = Rc::new(String);
    referencible::add_trait(runtime, module, None, &String).unwrap();

    // TODO String is not ToString. We could declare it on the struct, but that seems counterintuitive, no?
    //  Maybe a candidate for return self.strip().
    let mut ToString = Trait::new_with_self("ToString");
    let to_string_function = make_to_string_function(&ToString, &String);
    insert_functions(&mut ToString, [
        &to_string_function
    ].into_iter());
    let ToString = Rc::new(ToString);
    referencible::add_trait(runtime, module, None, &ToString).unwrap();

    let mut ConstructableByIntLiteral = Trait::new_with_self("ConstructableByIntLiteral");
    let parse_int_literal_function = FunctionPointer::new_global_function(
        "parse_int_literal",
        FunctionInterface::new_simple(
            [TypeProto::unit_struct(&String)].into_iter(),
            ConstructableByIntLiteral.create_generic_type("Self"),
        )
    );
    insert_functions(&mut ConstructableByIntLiteral, [
        &parse_int_literal_function
    ].into_iter());
    let ConstructableByIntLiteral = Rc::new(ConstructableByIntLiteral);
    referencible::add_trait(runtime, module, None, &ConstructableByIntLiteral).unwrap();


    let mut ConstructableByRealLiteral = Trait::new_with_self("ConstructableByRealLiteral");
    let parse_real_literal_function = FunctionPointer::new_global_function(
        "parse_real_literal",
        FunctionInterface::new_simple(
            [TypeProto::unit_struct(&String)].into_iter(),
            ConstructableByRealLiteral.create_generic_type("Self")
        ),
    );
    insert_functions(&mut ConstructableByRealLiteral, [
        &parse_real_literal_function
    ].into_iter());
    let ConstructableByRealLiteral = Rc::new(ConstructableByRealLiteral);
    referencible::add_trait(runtime, module, None, &ConstructableByRealLiteral).unwrap();


    let mut Real = Trait::new_with_self("Real");
    let float_functions = make_real_functions(&Real.create_generic_type("Self"));
    insert_functions(&mut Real, [
        &float_functions.pow,
        &float_functions.log
    ].into_iter());
    Real.add_simple_parent_requirement(&Number);
    Real.add_simple_parent_requirement(&ConstructableByRealLiteral);
    Real.add_simple_parent_requirement(&ConstructableByIntLiteral);
    let Real = Rc::new(Real);
    referencible::add_trait(runtime, module, None, &Real).unwrap();

    let mut Int = Trait::new_with_self("Int");
    Int.add_simple_parent_requirement(&Number);
    Int.add_simple_parent_requirement(&ConstructableByIntLiteral);
    let Int = Rc::new(Int);
    referencible::add_trait(runtime, module, None, &Int).unwrap();

    let mut Natural = Trait::new_with_self("Natural");
    Natural.add_simple_parent_requirement(&Int);
    let Natural = Rc::new(Natural);
    referencible::add_trait(runtime, module, None, &Natural).unwrap();

    Traits {
        Function,

        Eq,
        Eq_functions: eq_functions,

        Ord,
        Ord_functions: ord_functions,

        String,
        ToString,
        to_string_function,

        ConstructableByIntLiteral,
        parse_int_literal_function,
        ConstructableByRealLiteral,
        parse_real_literal_function,

        Number,
        Number_functions: number_functions,

        Real,
        Real_functions: float_functions,

        Int,
        Natural,
    }
}
