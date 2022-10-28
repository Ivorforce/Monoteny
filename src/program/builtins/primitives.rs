use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use uuid::Uuid;
use itertools::zip_eq;
use strum::IntoEnumIterator;
use crate::linker::scopes;
use crate::linker::scopes::Scope;
use crate::program::allocation::Reference;
use crate::program::builtins::traits;
use crate::program::builtins::traits::Traits;
use crate::program::functions::FunctionPointer;
use crate::program::primitives;
use crate::program::traits::{Trait, TraitConformanceDeclaration};
use crate::program::types::{TypeProto, TypeUnit};

pub struct Primitives {
    metatypes: HashMap<primitives::Type, Box<TypeProto>>,

    // logical
    pub and: Rc<FunctionPointer>,
    pub or: Rc<FunctionPointer>,
    pub not: Rc<FunctionPointer>,

    // eq
    pub equal_to: HashSet<Rc<FunctionPointer>>,
    pub not_equal_to: HashSet<Rc<FunctionPointer>>,

    // ord
    pub greater_than: HashSet<Rc<FunctionPointer>>,
    pub greater_than_or_equal_to: HashSet<Rc<FunctionPointer>>,
    pub lesser_than: HashSet<Rc<FunctionPointer>>,
    pub lesser_than_or_equal_to: HashSet<Rc<FunctionPointer>>,

    // number
    pub add: HashSet<Rc<FunctionPointer>>,
    pub subtract: HashSet<Rc<FunctionPointer>>,
    pub multiply: HashSet<Rc<FunctionPointer>>,
    pub divide: HashSet<Rc<FunctionPointer>>,

    pub positive: HashSet<Rc<FunctionPointer>>,
    pub negative: HashSet<Rc<FunctionPointer>>,

    pub modulo: HashSet<Rc<FunctionPointer>>,

    // float
    pub exponent: HashSet<Rc<FunctionPointer>>,
    pub logarithm: HashSet<Rc<FunctionPointer>>,

    // parse
    pub parse_int_literal: HashSet<Rc<FunctionPointer>>,
    pub parse_float_literal: HashSet<Rc<FunctionPointer>>,
}


pub fn make(mut constants: &mut Scope, traits: &Traits) -> Primitives {
    let bool_type = TypeProto::unit(TypeUnit::Primitive(primitives::Type::Bool));

    let mut add_function = |function: &Rc<FunctionPointer>, category: &mut HashSet<Rc<FunctionPointer>>, constants: &mut scopes::Scope| {
        category.insert(Rc::clone(&function));
        constants.overload_function(&function);
    };


    let mut primitive_metatypes = HashMap::new();

    let mut eq__ops: HashSet<Rc<FunctionPointer>> = HashSet::new();
    let mut neq_ops: HashSet<Rc<FunctionPointer>> = HashSet::new();

    let mut add_ops: HashSet<Rc<FunctionPointer>> = HashSet::new();
    let mut sub_ops: HashSet<Rc<FunctionPointer>> = HashSet::new();
    let mut mul_ops: HashSet<Rc<FunctionPointer>> = HashSet::new();
    let mut div_ops: HashSet<Rc<FunctionPointer>> = HashSet::new();

    let mut mod_ops: HashSet<Rc<FunctionPointer>> = HashSet::new();

    let mut exp_ops: HashSet<Rc<FunctionPointer>> = HashSet::new();
    let mut log_ops: HashSet<Rc<FunctionPointer>> = HashSet::new();

    let mut gr__ops: HashSet<Rc<FunctionPointer>> = HashSet::new();
    let mut geq_ops: HashSet<Rc<FunctionPointer>> = HashSet::new();
    let mut le__ops: HashSet<Rc<FunctionPointer>> = HashSet::new();
    let mut leq_ops: HashSet<Rc<FunctionPointer>> = HashSet::new();

    let mut pos_ops: HashSet<Rc<FunctionPointer>> = HashSet::new();
    let mut neg_ops: HashSet<Rc<FunctionPointer>> = HashSet::new();

    let mut parse_int_literal: HashSet<Rc<FunctionPointer>> = HashSet::new();
    let mut parse_float_literal: HashSet<Rc<FunctionPointer>> = HashSet::new();

    for primitive_type in primitives::Type::iter() {
        let type_ = &TypeProto::unit(TypeUnit::Primitive(primitive_type));
        let metatype = TypeProto::meta(type_.clone());

        primitive_metatypes.insert(primitive_type, metatype.clone());
        constants.insert_singleton(
            scopes::Environment::Global,
            Reference::make_immutable_type(metatype.clone()),
            &primitive_type.identifier_string()
        );

        // Pair-Associative
        let eq_functions = traits::make_eq_functions(type_);
        add_function(&eq_functions.equal_to, &mut eq__ops, &mut constants);
        add_function(&eq_functions.not_equal_to, &mut neq_ops, &mut constants);

        let eq_conformance = TraitConformanceDeclaration::make(
            &traits.Eq,
            vec![type_.clone()],
            vec![
                (&traits.Eq_functions.equal_to, &eq_functions.equal_to),
                (&traits.Eq_functions.not_equal_to, &eq_functions.not_equal_to),
            ]
        );
        constants.trait_conformance_declarations.add(&eq_conformance);

        if !primitive_type.is_number() {
            continue;
        }

        let number_functions = traits::make_number_functions(&type_);

        // Ord
        add_function(&number_functions.greater_than, &mut gr__ops, &mut constants);
        add_function(&number_functions.greater_than_or_equal_to, &mut geq_ops, &mut constants);
        add_function(&number_functions.lesser_than, &mut le__ops, &mut constants);
        add_function(&number_functions.lesser_than_or_equal_to, &mut leq_ops, &mut constants);

        let ord_conformance = TraitConformanceDeclaration::make_child(
            &traits.Ord, vec![&eq_conformance], vec![
                (&traits.Number_functions.greater_than, &number_functions.greater_than),
                (&traits.Number_functions.greater_than_or_equal_to, &number_functions.greater_than_or_equal_to),
                (&traits.Number_functions.lesser_than, &number_functions.lesser_than),
                (&traits.Number_functions.lesser_than_or_equal_to, &number_functions.lesser_than_or_equal_to),
            ]
        );
        constants.trait_conformance_declarations.add(&ord_conformance);

        // Number
        add_function(&number_functions.add, &mut add_ops, &mut constants);
        add_function(&number_functions.subtract, &mut sub_ops, &mut constants);
        add_function(&number_functions.multiply, &mut mul_ops, &mut constants);
        add_function(&number_functions.divide, &mut div_ops, &mut constants);
        add_function(&number_functions.modulo, &mut mod_ops, &mut constants);
        add_function(&number_functions.positive, &mut pos_ops, &mut constants);
        add_function(&number_functions.negative, &mut neg_ops, &mut constants);

        let _parse_int_literal = FunctionPointer::make_operator("parse_int_literal", 1, &TypeProto::unit(TypeUnit::Struct(Rc::clone(&traits.String))), &type_);
        add_function(&_parse_int_literal, &mut parse_int_literal, &mut constants);
        let ParseableByIntLiteral = TraitConformanceDeclaration::make(
            &traits.ConstructableByIntLiteral, vec![type_.clone()], vec![
                (&traits.parse_int_literal_function, &_parse_int_literal),
            ]
        );
        constants.trait_conformance_declarations.add(&ParseableByIntLiteral);

        let number_conformance = TraitConformanceDeclaration::make_child(
            &traits.Number, vec![&ord_conformance], vec![
                (&traits.Number_functions.add, &number_functions.add),
                (&traits.Number_functions.subtract, &number_functions.subtract),
                (&traits.Number_functions.multiply, &number_functions.multiply),
                (&traits.Number_functions.divide, &number_functions.divide),
                (&traits.Number_functions.positive, &number_functions.positive),
                (&traits.Number_functions.negative, &number_functions.negative),
                (&traits.Number_functions.modulo, &number_functions.modulo),
            ]
        );
        constants.trait_conformance_declarations.add(&number_conformance);

        if primitive_type.is_int() {
            constants.trait_conformance_declarations.add(
                &TraitConformanceDeclaration::make_child(&traits.Int, vec![&number_conformance, &ParseableByIntLiteral], vec![])
            );
        }

        if !(primitive_type.is_float()) {
            continue;
        }

        let float_functions = traits::make_float_functions(&type_);
        add_function(&float_functions.exponent, &mut exp_ops, &mut constants);
        add_function(&float_functions.logarithm, &mut log_ops, &mut constants);

        let _parse_float_literal = FunctionPointer::make_operator("parse_float_literal", 1, &TypeProto::unit(TypeUnit::Struct(Rc::clone(&traits.String))), &type_);
        add_function(&_parse_float_literal, &mut parse_float_literal, &mut constants);
        let ParseableByFloatLiteral = TraitConformanceDeclaration::make(
            &traits.ConstructableByFloatLiteral, vec![type_.clone()], vec![
                (&traits.parse_float_literal_function, &_parse_float_literal),
            ]
        );
        constants.trait_conformance_declarations.add(&ParseableByFloatLiteral);

        let float_conformance = TraitConformanceDeclaration::make_child(&traits.Float, vec![&number_conformance, &ParseableByIntLiteral, &ParseableByFloatLiteral], vec![
            (&traits.Float_functions.exponent, &float_functions.exponent),
            (&traits.Float_functions.logarithm, &float_functions.logarithm),
        ]);

        constants.trait_conformance_declarations.add(&float_conformance);
    }

    let and_op = FunctionPointer::make_operator("and_f", 2, &bool_type, &bool_type);
    constants.overload_function(&and_op);

    let or__op = FunctionPointer::make_operator("or_f", 2, &bool_type, &bool_type);
    constants.overload_function(&or__op);

    let not_op = FunctionPointer::make_operator("not_f", 1, &bool_type, &bool_type);
    constants.overload_function(&not_op);


    Primitives {
        metatypes: primitive_metatypes,

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
