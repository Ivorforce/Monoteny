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
}


pub fn make(mut constants: &mut Scope, traits: &Traits) -> Primitives {
    let bool_type = TypeProto::unit(TypeUnit::Primitive(primitives::Type::Bool));

    let mut add_function = |function: &Rc<FunctionPointer>, category: &mut HashSet<Rc<FunctionPointer>>, constants: &mut scopes::Scope| {
        category.insert(Rc::clone(&function));
        constants.overload_function(&function);
    };

    let make_conformance_declaration = |trait_: &Rc<Trait>, parent_conformance: &Rc<TraitConformanceDeclaration>, function_implementations: Vec<(&Rc<FunctionPointer>, &Rc<FunctionPointer>)>| -> Rc<TraitConformanceDeclaration> {
        Rc::new(TraitConformanceDeclaration {
            id: Uuid::new_v4(),
            trait_: Rc::clone(trait_),
            arguments: parent_conformance.arguments.clone(),
            requirements: HashSet::new(),
            trait_requirements_conformance: zip_eq(trait_.requirements.iter().map(Rc::clone), [parent_conformance].map(Rc::clone)).collect(),
            function_implementations: function_implementations.into_iter()
                .map(|(l, r)| (Rc::clone(l), Rc::clone(r)))
                .collect()
        })
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

    for primitive_type in primitives::Type::iter() {
        let type_ = &TypeProto::unit(TypeUnit::Primitive(primitive_type));
        let metatype = TypeProto::meta(type_.clone());

        primitive_metatypes.insert(primitive_type, metatype.clone());
        constants.insert_singleton(
            scopes::Environment::Global,
            Reference::make_immutable(metatype.clone()),
            &primitive_type.identifier_string()
        );

        // Pair-Associative
        let eq_functions = traits::make_eq_functions(type_);
        add_function(&eq_functions.equal_to, &mut eq__ops, &mut constants);
        add_function(&eq_functions.not_equal_to, &mut neq_ops, &mut constants);

        let eq_conformance = Rc::new(TraitConformanceDeclaration {
            id: Uuid::new_v4(),
            trait_: Rc::clone(&traits.Eq),
            arguments: vec![type_.clone()],
            requirements: HashSet::new(),
            trait_requirements_conformance: HashMap::new(),
            function_implementations: HashMap::from([
                (Rc::clone(&traits.Eq_functions.equal_to), Rc::clone(&eq_functions.equal_to)),
                (Rc::clone(&traits.Eq_functions.not_equal_to), Rc::clone(&eq_functions.not_equal_to)),
            ])
        });
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

        let ord_conformance = make_conformance_declaration(
            &traits.Ord, &eq_conformance, vec![
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

        let number_conformance = make_conformance_declaration(
            &traits.Number, &ord_conformance, vec![
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
                &make_conformance_declaration(&traits.Int, &number_conformance, vec![])
            );
        }

        if !(primitive_type.is_float()) {
            continue;
        }

        let float_functions = traits::make_float_functions(&type_);
        add_function(&float_functions.exponent, &mut exp_ops, &mut constants);
        add_function(&float_functions.logarithm, &mut log_ops, &mut constants);

        let float_conformance = make_conformance_declaration(&traits.Float, &number_conformance, vec![
            (&traits.Float_functions.exponent, &float_functions.exponent),
            (&traits.Float_functions.logarithm, &float_functions.logarithm),
        ]);

        constants.trait_conformance_declarations.add(&float_conformance);
    }

    let and_op = FunctionPointer::make_operator("&&", "and", 2, &bool_type, &bool_type);
    constants.overload_function(&and_op);

    let or__op = FunctionPointer::make_operator("||", "or", 2, &bool_type, &bool_type);
    constants.overload_function(&or__op);

    let not_op = FunctionPointer::make_operator("!", "not", 1, &bool_type, &bool_type);
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
    }
}
