use std::collections::HashMap;
use std::rc::Rc;
use strum::IntoEnumIterator;
use crate::program::builtins::traits;
use crate::program::builtins::traits::Traits;
use crate::program::functions::{FunctionInterface, FunctionPointer};
use crate::program::module::Module;
use crate::program::primitives;
use crate::program::traits::Trait;
use crate::program::types::{TypeProto, TypeUnit};


pub struct PrimitiveFunctions {
    // logical
    pub and: Rc<FunctionPointer>,
    pub or: Rc<FunctionPointer>,
    pub not: Rc<FunctionPointer>,

    // eq
    pub equal_to: HashMap<primitives::Type, Rc<FunctionPointer>>,
    pub not_equal_to: HashMap<primitives::Type, Rc<FunctionPointer>>,

    // ord
    pub greater_than: HashMap<primitives::Type, Rc<FunctionPointer>>,
    pub greater_than_or_equal_to: HashMap<primitives::Type, Rc<FunctionPointer>>,
    pub lesser_than: HashMap<primitives::Type, Rc<FunctionPointer>>,
    pub lesser_than_or_equal_to: HashMap<primitives::Type, Rc<FunctionPointer>>,

    // number
    pub add: HashMap<primitives::Type, Rc<FunctionPointer>>,
    pub subtract: HashMap<primitives::Type, Rc<FunctionPointer>>,
    pub multiply: HashMap<primitives::Type, Rc<FunctionPointer>>,
    pub divide: HashMap<primitives::Type, Rc<FunctionPointer>>,
    pub modulo: HashMap<primitives::Type, Rc<FunctionPointer>>,

    pub positive: HashMap<primitives::Type, Rc<FunctionPointer>>,
    pub negative: HashMap<primitives::Type, Rc<FunctionPointer>>,  // TODO This shouldn't exist for unsigned types

    // float
    pub exponent: HashMap<primitives::Type, Rc<FunctionPointer>>,
    pub logarithm: HashMap<primitives::Type, Rc<FunctionPointer>>,

    // parse
    pub parse_int_literal: HashMap<primitives::Type, Rc<FunctionPointer>>,
    pub parse_float_literal: HashMap<primitives::Type, Rc<FunctionPointer>>,
}

pub fn create_traits(module: &mut Module) -> HashMap<primitives::Type, Rc<Trait>> {
    let mut traits: HashMap<primitives::Type, Rc<Trait>> = Default::default();

    for primitive_type in primitives::Type::iter() {
        let trait_ = Rc::new(Trait::new(primitive_type.identifier_string()));
        module.add_trait(&trait_);
        traits.insert(primitive_type, trait_);
    }

    traits
}

pub fn create_functions(module: &mut Module, traits: &Traits, basis: &HashMap<primitives::Type, Rc<Trait>>) -> PrimitiveFunctions {
    let bool_type = TypeProto::simple_struct(&basis[&primitives::Type::Bool]);

    let mut add_function = |function: &Rc<FunctionPointer>, primitive_type: primitives::Type, category: &mut HashMap<primitives::Type, Rc<FunctionPointer>>, module: &mut Module| {
        module.add_function(&function);
        category.insert(primitive_type, Rc::clone(&function));
    };


    let mut eq__ops: HashMap<primitives::Type, Rc<FunctionPointer>> = HashMap::new();
    let mut neq_ops: HashMap<primitives::Type, Rc<FunctionPointer>> = HashMap::new();

    let mut add_ops: HashMap<primitives::Type, Rc<FunctionPointer>> = HashMap::new();
    let mut sub_ops: HashMap<primitives::Type, Rc<FunctionPointer>> = HashMap::new();
    let mut mul_ops: HashMap<primitives::Type, Rc<FunctionPointer>> = HashMap::new();
    let mut div_ops: HashMap<primitives::Type, Rc<FunctionPointer>> = HashMap::new();

    let mut mod_ops: HashMap<primitives::Type, Rc<FunctionPointer>> = HashMap::new();

    let mut exp_ops: HashMap<primitives::Type, Rc<FunctionPointer>> = HashMap::new();
    let mut log_ops: HashMap<primitives::Type, Rc<FunctionPointer>> = HashMap::new();

    let mut gr__ops: HashMap<primitives::Type, Rc<FunctionPointer>> = HashMap::new();
    let mut geq_ops: HashMap<primitives::Type, Rc<FunctionPointer>> = HashMap::new();
    let mut le__ops: HashMap<primitives::Type, Rc<FunctionPointer>> = HashMap::new();
    let mut leq_ops: HashMap<primitives::Type, Rc<FunctionPointer>> = HashMap::new();

    let mut pos_ops: HashMap<primitives::Type, Rc<FunctionPointer>> = HashMap::new();
    let mut neg_ops: HashMap<primitives::Type, Rc<FunctionPointer>> = HashMap::new();

    let mut parse_int_literal: HashMap<primitives::Type, Rc<FunctionPointer>> = HashMap::new();
    let mut parse_float_literal: HashMap<primitives::Type, Rc<FunctionPointer>> = HashMap::new();

    for (primitive_type, trait_) in basis.iter() {
        let type_ = TypeProto::simple_struct(&basis[primitive_type]);
        let primitive_type = *primitive_type;

        // Pair-Associative
        let eq_functions = traits::make_eq_functions(&type_, &bool_type);
        add_function(&eq_functions.equal_to, primitive_type, &mut eq__ops, module);
        add_function(&eq_functions.not_equal_to, primitive_type, &mut neq_ops, module);

        module.trait_conformance.add_conformance_manual(
            traits.Eq.create_generic_binding(vec![(&"self".into(), type_.clone())]),
            vec![
                (&traits.Eq_functions.equal_to, &eq_functions.equal_to),
                (&traits.Eq_functions.not_equal_to, &eq_functions.not_equal_to),
            ]
        ).unwrap();

        if !primitive_type.is_number() {
            continue;
        }

        // Ord
        let ord_functions = traits::make_ord_functions(&type_, &bool_type);
        add_function(&ord_functions.greater_than, primitive_type, &mut gr__ops, module);
        add_function(&ord_functions.greater_than_or_equal_to, primitive_type, &mut geq_ops, module);
        add_function(&ord_functions.lesser_than, primitive_type, &mut le__ops, module);
        add_function(&ord_functions.lesser_than_or_equal_to, primitive_type, &mut leq_ops, module);

        module.trait_conformance.add_conformance_manual(
            traits.Ord.create_generic_binding(vec![(&"self".into(), type_.clone())]),
            vec![
                (&traits.Ord_functions.greater_than, &ord_functions.greater_than),
                (&traits.Ord_functions.greater_than_or_equal_to, &ord_functions.greater_than_or_equal_to),
                (&traits.Ord_functions.lesser_than, &ord_functions.lesser_than),
                (&traits.Ord_functions.lesser_than_or_equal_to, &ord_functions.lesser_than_or_equal_to),
            ]
        ).unwrap();

        // Number
        let number_functions = traits::make_number_functions(&type_, &bool_type);
        add_function(&number_functions.add, primitive_type, &mut add_ops, module);
        add_function(&number_functions.subtract, primitive_type, &mut sub_ops, module);
        add_function(&number_functions.multiply, primitive_type, &mut mul_ops, module);
        add_function(&number_functions.divide, primitive_type, &mut div_ops, module);
        add_function(&number_functions.modulo, primitive_type, &mut mod_ops, module);
        add_function(&number_functions.positive, primitive_type, &mut pos_ops, module);
        add_function(&number_functions.negative, primitive_type, &mut neg_ops, module);

        let _parse_int_literal = FunctionPointer::new_global(
            "parse_int_literal",
            FunctionInterface::new_operator(1, &TypeProto::unit(TypeUnit::Struct(Rc::clone(&traits.String))), &type_)
        );
        add_function(&_parse_int_literal, primitive_type, &mut parse_int_literal, module);
        module.trait_conformance.add_conformance_manual(
            traits.ConstructableByIntLiteral.create_generic_binding(vec![(&"self".into(), type_.clone())]),
            vec![
                (&traits.parse_int_literal_function, &_parse_int_literal),
            ]
        ).unwrap();

        module.trait_conformance.add_conformance_manual(
            traits.Number.create_generic_binding(vec![(&"self".into(), type_.clone())]),
            vec![
                (&traits.Number_functions.add, &number_functions.add),
                (&traits.Number_functions.subtract, &number_functions.subtract),
                (&traits.Number_functions.multiply, &number_functions.multiply),
                (&traits.Number_functions.divide, &number_functions.divide),
                (&traits.Number_functions.modulo, &number_functions.modulo),
                (&traits.Number_functions.positive, &number_functions.positive),
                (&traits.Number_functions.negative, &number_functions.negative),
            ]
        ).unwrap();

        if primitive_type.is_int() {
            module.trait_conformance.add_conformance_manual(traits.Int.create_generic_binding(vec![(&"self".into(), type_.clone())]), vec![]).unwrap();
        }

        if !(primitive_type.is_float()) {
            continue;
        }

        let float_functions = traits::make_float_functions(&type_);
        add_function(&float_functions.exponent, primitive_type, &mut exp_ops, module);
        add_function(&float_functions.logarithm, primitive_type, &mut log_ops, module);

        let _parse_float_literal = FunctionPointer::new_global(
            "parse_float_literal",
            FunctionInterface::new_operator(1, &TypeProto::unit(TypeUnit::Struct(Rc::clone(&traits.String))), &type_)
        );
        add_function(&_parse_float_literal, primitive_type, &mut parse_float_literal, module);
        module.trait_conformance.add_conformance_manual(
            traits.ConstructableByFloatLiteral.create_generic_binding(vec![(&"self".into(), type_.clone())]), vec![
                (&traits.parse_float_literal_function, &_parse_float_literal),
            ]
        ).unwrap();

        module.trait_conformance.add_conformance_manual(
            traits.Float.create_generic_binding(vec![(&"self".into(), type_)]), vec![
            (&traits.Float_functions.exponent, &float_functions.exponent),
            (&traits.Float_functions.logarithm, &float_functions.logarithm),
        ]).unwrap();
    }

    let and_op = FunctionPointer::new_global(
        "and_f",
        FunctionInterface::new_operator(2, &bool_type, &bool_type)
    );
    module.add_function(&and_op);

    let or__op = FunctionPointer::new_global(
        "or_f",
        FunctionInterface::new_operator(2, &bool_type, &bool_type)
    );
    module.add_function(&or__op);

    let not_op = FunctionPointer::new_global(
        "not_f",
        FunctionInterface::new_operator(1, &bool_type, &bool_type)
    );
    module.add_function(&not_op);


    PrimitiveFunctions {
        and: and_op,
        or: or__op,

        equal_to: eq__ops,
        not_equal_to: neq_ops,

        greater_than: gr__ops,
        greater_than_or_equal_to: geq_ops,
        lesser_than: le__ops,
        lesser_than_or_equal_to: leq_ops,

        add: add_ops,
        subtract: sub_ops,
        multiply: mul_ops,
        divide: div_ops,
        modulo: mod_ops,

        exponent: exp_ops,
        logarithm: log_ops,

        positive: pos_ops,
        negative: neg_ops,
        not: not_op,

        parse_int_literal,
        parse_float_literal,
    }
}
