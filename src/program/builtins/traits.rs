use std::fmt::Debug;
use std::rc::Rc;

use crate::interpreter::runtime::Runtime;
use crate::program::functions::{FunctionHead, FunctionInterface, FunctionLogic, FunctionLogicDescriptor, FunctionRepresentation};
use crate::program::module::Module;
use crate::program::primitives;
use crate::program::traits::{Trait, TraitConformanceRule};
use crate::program::types::TypeProto;
use crate::resolver::referencible;

pub fn insert_functions<'a, I>(trait_: &mut Trait, functions: I) where I: Iterator<Item=&'a Rc<FunctionHead>> {
    for ptr in functions {
        trait_.abstract_functions.insert(Rc::clone(ptr));
    }
}

#[allow(non_snake_case)]
#[derive(Clone)]
pub struct Traits {
    /// Supertype of all objects.
    pub Any: Rc<Trait>,
    pub Any_functions: AnyFunctions,

    /// Supertype of all function objects.
    /// No requirements yet (will require call_as_function to return self!).
    pub Function: Rc<Trait>,

    pub Eq: Rc<Trait>,
    pub Eq_functions: EqFunctions,

    pub Ord: Rc<Trait>,
    pub Ord_functions: OrdFunctions,

    pub String: Rc<Trait>,
    pub ToString: Rc<Trait>,
    pub to_string_function: Rc<FunctionHead>,

    pub ConstructableByIntLiteral: Rc<Trait>,
    pub parse_int_literal_function: Rc<FunctionHead>,

    pub ConstructableByRealLiteral: Rc<Trait>,
    pub parse_real_literal_function: Rc<FunctionHead>,

    pub Number: Rc<Trait>,
    pub Number_functions: NumberFunctions,

    pub Real: Rc<Trait>,
    pub Real_functions: RealFunctions,

    pub Int: Rc<Trait>,
    pub Natural: Rc<Trait>,
}

#[derive(Clone)]
pub struct AnyFunctions {
    pub clone: Rc<FunctionHead>,
}

pub fn make_any_functions(type_: &Rc<TypeProto>) -> AnyFunctions {
    AnyFunctions {
        clone: FunctionHead::new_static(
            FunctionInterface::new_member(
                type_.clone(),
                [].into_iter(),
                type_.clone()
            ),
            FunctionRepresentation::new_member_function("clone")
        ),
    }
}

#[derive(Clone)]
pub struct EqFunctions {
    pub equal_to: Rc<FunctionHead>,
    pub not_equal_to: Rc<FunctionHead>,
}

pub fn make_eq_functions(type_: &Rc<TypeProto>, bool_type: &Rc<TypeProto>) -> EqFunctions {
    EqFunctions {
        equal_to: FunctionHead::new_static(
            FunctionInterface::new_operator(2, type_, bool_type),
            FunctionRepresentation::new_global_function("is_equal"),
        ),
        not_equal_to: FunctionHead::new_static(
            FunctionInterface::new_operator(2, type_, bool_type),
            FunctionRepresentation::new_global_function("is_not_equal"),
        ),
    }
}

#[derive(Clone)]
pub struct OrdFunctions {
    pub greater_than: Rc<FunctionHead>,
    pub greater_than_or_equal_to: Rc<FunctionHead>,
    pub lesser_than: Rc<FunctionHead>,
    pub lesser_than_or_equal_to: Rc<FunctionHead>,
}

pub fn make_ord_functions(type_: &Rc<TypeProto>, bool_type: &Rc<TypeProto>) -> OrdFunctions {
    OrdFunctions {
        greater_than: FunctionHead::new_static(
            FunctionInterface::new_operator(2, type_, bool_type),
            FunctionRepresentation::new_global_function("is_greater"),
        ),
        greater_than_or_equal_to: FunctionHead::new_static(
            FunctionInterface::new_operator(2, type_, bool_type),
            FunctionRepresentation::new_global_function("is_greater_or_equal"),
        ),
        lesser_than: FunctionHead::new_static(
            FunctionInterface::new_operator(2, type_, bool_type),
            FunctionRepresentation::new_global_function("is_lesser"),
        ),
        lesser_than_or_equal_to: FunctionHead::new_static(
            FunctionInterface::new_operator(2, type_, bool_type),
            FunctionRepresentation::new_global_function("is_lesser_or_equal"),
        ),
    }
}

#[derive(Clone)]
pub struct NumberFunctions {
    pub add: Rc<FunctionHead>,
    pub subtract: Rc<FunctionHead>,
    pub multiply: Rc<FunctionHead>,
    pub divide: Rc<FunctionHead>,

    pub modulo: Rc<FunctionHead>,

    /// You may argue that unsigned numbers should not need to support negative.
    /// However, all unsigned numbers have rollover. That means that e.g. -1 = MAX, and
    ///  it's generally a perfectly valid operation.
    pub negative: Rc<FunctionHead>,
}

pub fn make_number_functions(type_: &Rc<TypeProto>) -> NumberFunctions {
    NumberFunctions {
        add: FunctionHead::new_static(
            FunctionInterface::new_operator(2, type_, type_),
            FunctionRepresentation::new_global_function("add"),
        ),
        subtract: FunctionHead::new_static(
            FunctionInterface::new_operator(2, type_, type_),
            FunctionRepresentation::new_global_function("subtract"),
        ),
        multiply: FunctionHead::new_static(
            FunctionInterface::new_operator(2, type_, type_),
            FunctionRepresentation::new_global_function("multiply"),
        ),
        divide: FunctionHead::new_static(
            FunctionInterface::new_operator(2, type_, type_),
            FunctionRepresentation::new_global_function("divide"),
        ),

        negative: FunctionHead::new_static(
            FunctionInterface::new_operator(1, type_, type_),
            FunctionRepresentation::new_global_function("negative"),
        ),

        modulo: FunctionHead::new_static(
            FunctionInterface::new_operator(2, type_, type_),
            FunctionRepresentation::new_global_function("modulo"),
        ),
    }
}

#[derive(Clone)]
pub struct RealFunctions {
    pub pow: Rc<FunctionHead>,
    pub log: Rc<FunctionHead>,
}

pub fn make_real_functions(type_: &Rc<TypeProto>) -> RealFunctions {
    RealFunctions {
        pow: FunctionHead::new_static(
            FunctionInterface::new_operator(2, type_, type_),
            FunctionRepresentation::new_global_function("pow"),
        ),
        log: FunctionHead::new_static(
            FunctionInterface::new_operator(1, type_, type_),
            FunctionRepresentation::new_global_function("log"),
        ),
    }
}

#[allow(non_snake_case)]
pub fn make_to_string_function(type_: &Trait, String: &Rc<Trait>) -> Rc<FunctionHead> {
    FunctionHead::new_static(
        FunctionInterface::new_member(
            type_.create_generic_type("Self"),
            [].into_iter(),
            TypeProto::unit_struct(&String)
        ),
        FunctionRepresentation::new_member_function("to_string"),
    )
}

#[allow(non_snake_case)]
pub fn create(runtime: &mut Runtime, module: &mut Module) -> Traits {
    let primitive_traits = runtime.primitives.as_ref().unwrap();
    let bool_type = TypeProto::unit_struct(&primitive_traits[&primitives::Type::Bool]);

    let mut Any = Trait::new_with_self("Any");
    let any_functions = make_any_functions(&Any.create_generic_type("Self"));
    insert_functions(&mut Any, [
        &any_functions.clone,
    ].into_iter());
    let Any = Rc::new(Any);
    referencible::add_trait(runtime, module, None, &Any).unwrap();

    let mut Function = Trait::new_with_self("Function");
    Function.add_simple_parent_requirement(&Any);
    let Function = Rc::new(Function);
    referencible::add_trait(runtime, module, None, &Function).unwrap();

    let mut Eq = Trait::new_with_self("Eq");
    let eq_functions = make_eq_functions(&Eq.create_generic_type("Self"), &bool_type);
    insert_functions(&mut Eq, [
        &eq_functions.equal_to,
        &eq_functions.not_equal_to,
    ].into_iter());
    Eq.add_simple_parent_requirement(&Any);
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
    Ord.add_simple_parent_requirement(&Any);
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
    Number.add_simple_parent_requirement(&Any);
    Number.add_simple_parent_requirement(&Ord);
    let Number = Rc::new(Number);
    referencible::add_trait(runtime, module, None, &Number).unwrap();

    let mut String = Trait::new_with_self("String");
    String.add_simple_parent_requirement(&Any);
    let String = Rc::new(String);
    referencible::add_trait(runtime, module, None, &String).unwrap();

    // TODO String is not ToString. We could declare it on the struct, but that seems counterintuitive, no?
    //  Maybe a candidate for return self.strip().
    let mut ToString = Trait::new_with_self("ToString");
    let to_string_function = make_to_string_function(&ToString, &String);
    insert_functions(&mut ToString, [
        &to_string_function
    ].into_iter());
    ToString.add_simple_parent_requirement(&Any);
    let ToString = Rc::new(ToString);
    referencible::add_trait(runtime, module, None, &ToString).unwrap();

    let mut ConstructableByIntLiteral = Trait::new_with_self("ConstructableByIntLiteral");
    let parse_int_literal_function = FunctionHead::new_static(
        FunctionInterface::new_simple(
            [TypeProto::unit_struct(&String)].into_iter(),
            ConstructableByIntLiteral.create_generic_type("Self"),
        ),
        FunctionRepresentation::new_global_function("parse_int_literal"),
    );
    insert_functions(&mut ConstructableByIntLiteral, [
        &parse_int_literal_function
    ].into_iter());
    ConstructableByIntLiteral.add_simple_parent_requirement(&Any);
    let ConstructableByIntLiteral = Rc::new(ConstructableByIntLiteral);
    referencible::add_trait(runtime, module, None, &ConstructableByIntLiteral).unwrap();


    let mut ConstructableByRealLiteral = Trait::new_with_self("ConstructableByRealLiteral");
    let parse_real_literal_function = FunctionHead::new_static(
        FunctionInterface::new_simple(
            [TypeProto::unit_struct(&String)].into_iter(),
            ConstructableByRealLiteral.create_generic_type("Self")
        ),
        FunctionRepresentation::new_global_function("parse_real_literal"),
    );
    insert_functions(&mut ConstructableByRealLiteral, [
        &parse_real_literal_function
    ].into_iter());
    ConstructableByRealLiteral.add_simple_parent_requirement(&Any);
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
    Real.add_simple_parent_requirement(&Any);
    let Real = Rc::new(Real);
    referencible::add_trait(runtime, module, None, &Real).unwrap();

    let mut Int = Trait::new_with_self("Int");
    Int.add_simple_parent_requirement(&Number);
    Int.add_simple_parent_requirement(&ConstructableByIntLiteral);
    Int.add_simple_parent_requirement(&Any);
    let Int = Rc::new(Int);
    referencible::add_trait(runtime, module, None, &Int).unwrap();

    let mut Natural = Trait::new_with_self("Natural");
    Natural.add_simple_parent_requirement(&Any);
    Natural.add_simple_parent_requirement(&Int);
    let Natural = Rc::new(Natural);
    referencible::add_trait(runtime, module, None, &Natural).unwrap();

    Traits {
        Any,
        Any_functions: any_functions,

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

pub fn create_functions(runtime: &mut Runtime, module: &mut Module) {
    // TODO Cloning is dumb but we can't hold a runtime reference.
    //  It's not too bad because it's all Rcs though.
    let traits = runtime.traits.as_ref().unwrap().clone();
    // let primitive_traits = runtime.primitives.as_ref().unwrap().clone();
    // let bool_type = TypeProto::unit_struct(&primitive_traits[&primitives::Type::Bool]);

    let string_type = TypeProto::unit_struct(&traits.String);
    let any_functions = make_any_functions(&string_type);
    runtime.source.fn_logic.insert(
        Rc::clone(&any_functions.clone),
        FunctionLogic::Descriptor(FunctionLogicDescriptor::Clone(string_type.clone())),
    );
    referencible::add_function(runtime, module, None, &any_functions.clone).unwrap();

    module.trait_conformance.add_conformance_rule(TraitConformanceRule::manual(
        traits.Any.create_generic_binding(vec![("Self", string_type)]),
        vec![
            (&traits.Any_functions.clone, &any_functions.clone),
        ]
    ));
}
