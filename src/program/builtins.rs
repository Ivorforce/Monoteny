use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use itertools::zip_eq;
use uuid::Uuid;
use strum::IntoEnumIterator;
use precedence::PrecedenceGroups;
use crate::linker::scopes;
use crate::program::traits::{Trait, TraitConformanceDeclaration, TraitConformanceRequirement};
use crate::parser;
use crate::linker::precedence::{OperatorAssociativity, PrecedenceGroup};
use crate::program::types::*;
use crate::program::primitives;
use crate::program;
use crate::program::allocation::Reference;
use crate::program::functions::{FunctionForm, FunctionPointer, HumanFunctionInterface, MachineFunctionInterface};
use crate::program::structs::Struct;

pub mod precedence;
pub mod debug;
pub mod strings;

pub struct Builtins {
    pub traits: Traits,
    pub operators: Operators,
    pub debug: debug::Functions,
    pub primitive_metatypes: HashMap<primitives::Type, Box<TypeProto>>,
    pub strings: strings::Strings,
    pub precedence_groups: PrecedenceGroups,

    pub global_constants: scopes::Scope<'static>,
}

pub struct Operators {
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
    pub exponentiate: HashSet<Rc<FunctionPointer>>,
}

#[allow(non_snake_case)]
pub struct Traits {
    pub all: HashSet<Rc<Trait>>,

    pub Eq: Rc<Trait>,
    pub Ord: Rc<Trait>,

    pub Number: Rc<Trait>,
    pub Float: Rc<Trait>,
    pub Int: Rc<Trait>,
}

pub struct EqFunctions {
    pub equal_to: Rc<FunctionPointer>,
    pub not_equal_to: Rc<FunctionPointer>,
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

pub fn make_eq_functions(type_: &Box<TypeProto>) -> EqFunctions {
    let bool_type = TypeProto::unit(TypeUnit::Primitive(primitives::Type::Bool));

    EqFunctions {
        equal_to: FunctionPointer::make_operator("==", "is_equal", 2, type_, &bool_type),
        not_equal_to: FunctionPointer::make_operator("!=", "is_not_equal", 2, type_, &bool_type),
    }
}

pub fn make_number_functions(type_: &Box<TypeProto>) -> NumberFunctions {
    let bool_type = TypeProto::unit(TypeUnit::Primitive(primitives::Type::Bool));

    NumberFunctions {
        add: FunctionPointer::make_operator("+", "add", 2, type_, type_),
        subtract: FunctionPointer::make_operator("-", "subtract", 2, type_, type_),
        multiply: FunctionPointer::make_operator("*", "multiply", 2, type_, type_),
        divide: FunctionPointer::make_operator("/", "divide", 2, type_, type_),

        positive: FunctionPointer::make_operator("+", "positive", 1, type_, type_),
        negative: FunctionPointer::make_operator("-", "negative", 1, type_, type_),

        modulo: FunctionPointer::make_operator("%", "modulo", 2, type_, type_),

        greater_than: FunctionPointer::make_operator(">", "is_greater", 2, type_, &bool_type),
        greater_than_or_equal_to: FunctionPointer::make_operator(">=", "is_greater_or_equal", 2, type_, &bool_type),
        lesser_than: FunctionPointer::make_operator("<", "is_lesser", 2, type_, &bool_type),
        lesser_than_or_equal_to: FunctionPointer::make_operator("<=", "is_lesser_or_equal", 2, type_, &bool_type),
    }
}

pub fn create_builtins() -> Rc<Builtins> {
    let mut constants: scopes::Scope = scopes::Scope::new();

    let bool_type = TypeProto::unit(TypeUnit::Primitive(primitives::Type::Bool));
    let generic_id = Uuid::new_v4();
    let generic_type = TypeProto::unit(TypeUnit::Any(generic_id));


    let precedence_groups = precedence::make_groups(&mut constants);


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

    let mut exp_ops: HashSet<Rc<FunctionPointer>> = HashSet::new();
    let mut mod_ops: HashSet<Rc<FunctionPointer>> = HashSet::new();

    let mut gr__ops: HashSet<Rc<FunctionPointer>> = HashSet::new();
    let mut geq_ops: HashSet<Rc<FunctionPointer>> = HashSet::new();
    let mut le__ops: HashSet<Rc<FunctionPointer>> = HashSet::new();
    let mut leq_ops: HashSet<Rc<FunctionPointer>> = HashSet::new();

    let mut pos_ops: HashSet<Rc<FunctionPointer>> = HashSet::new();
    let mut neg_ops: HashSet<Rc<FunctionPointer>> = HashSet::new();

    let abstract_eq_functions = make_eq_functions(&generic_type);
    let eq_trait = make_trait("Eq", &generic_id, vec![
        &abstract_eq_functions.equal_to,
        &abstract_eq_functions.not_equal_to,
    ], vec![]);
    constants.insert_trait(&eq_trait);

    let abstract_number_functions = make_number_functions(&generic_type);

    let ord_trait = make_trait("Ord", &generic_id, vec![
        &abstract_number_functions.greater_than,
        &abstract_number_functions.greater_than_or_equal_to,
        &abstract_number_functions.lesser_than,
        &abstract_number_functions.lesser_than_or_equal_to,
    ], vec![Rc::clone(&eq_trait)]);
    constants.insert_trait(&ord_trait);

    let number_trait = make_trait("Number", &generic_id, vec![
        &abstract_number_functions.add,
        &abstract_number_functions.subtract,
        &abstract_number_functions.multiply,
        &abstract_number_functions.divide,

        &abstract_number_functions.positive,
        &abstract_number_functions.negative,

        &abstract_number_functions.modulo,
    ], vec![Rc::clone(&ord_trait)]);
    constants.insert_trait(&number_trait);

    let float_trait = make_trait("Float", &generic_id, vec![], vec![Rc::clone(&number_trait)]);
    constants.insert_trait(&float_trait);

    let int_trait = make_trait("Int", &generic_id, vec![], vec![Rc::clone(&number_trait)]);
    constants.insert_trait(&int_trait);

    let traits = Traits {
        all: [&eq_trait, &ord_trait, &number_trait, &float_trait, &int_trait].map(Rc::clone).into_iter().collect(),
        Eq: eq_trait,
        Ord: ord_trait,
        Number: number_trait,
        Float: float_trait,
        Int: int_trait,
    };


    let mut add_function = |function: &Rc<FunctionPointer>, category: &mut HashSet<Rc<FunctionPointer>>, constants: &mut scopes::Scope| {
        category.insert(Rc::clone(&function));
        constants.overload_function(&function);
    };

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
        let eq_functions = make_eq_functions(type_);
        add_function(&eq_functions.equal_to, &mut eq__ops, &mut constants);
        add_function(&eq_functions.not_equal_to, &mut neq_ops, &mut constants);

        let eq_conformance = Rc::new(TraitConformanceDeclaration {
            id: Uuid::new_v4(),
            trait_: Rc::clone(&traits.Eq),
            arguments: vec![type_.clone()],
            requirements: HashSet::new(),
            trait_requirements_conformance: HashMap::new(),
            function_implementations: HashMap::from([
                (Rc::clone(&abstract_eq_functions.equal_to), Rc::clone(&eq_functions.equal_to)),
                (Rc::clone(&abstract_eq_functions.not_equal_to), Rc::clone(&eq_functions.not_equal_to)),
            ])
        });
        constants.trait_conformance_declarations.add(&eq_conformance);

        if !primitive_type.is_number() {
            continue;
        }

        let number_functions = make_number_functions(&type_);

        // Ord
        add_function(&number_functions.greater_than, &mut gr__ops, &mut constants);
        add_function(&number_functions.greater_than_or_equal_to, &mut geq_ops, &mut constants);
        add_function(&number_functions.lesser_than, &mut le__ops, &mut constants);
        add_function(&number_functions.lesser_than_or_equal_to, &mut leq_ops, &mut constants);

        let ord_conformance = make_conformance_declaration(
            &traits.Ord, &eq_conformance, vec![
                (&abstract_number_functions.greater_than, &number_functions.greater_than),
                (&abstract_number_functions.greater_than_or_equal_to, &number_functions.greater_than_or_equal_to),
                (&abstract_number_functions.lesser_than, &number_functions.lesser_than),
                (&abstract_number_functions.lesser_than_or_equal_to, &number_functions.lesser_than_or_equal_to),
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
                (&abstract_number_functions.add, &number_functions.add),
                (&abstract_number_functions.subtract, &number_functions.subtract),
                (&abstract_number_functions.multiply, &number_functions.multiply),
                (&abstract_number_functions.divide, &number_functions.divide),

                (&abstract_number_functions.positive, &number_functions.positive),
                (&abstract_number_functions.negative, &number_functions.negative),

                (&abstract_number_functions.modulo, &number_functions.modulo),
            ]
        );
        constants.trait_conformance_declarations.add(&number_conformance);

        if primitive_type.is_float() {
            let exp_op = FunctionPointer::make_operator("**", "exponentiate", 2, type_, type_);
            constants.overload_function(&exp_op);
            exp_ops.insert(Rc::clone(&exp_op));

            constants.trait_conformance_declarations.add(
                &make_conformance_declaration(&traits.Float, &number_conformance, vec![])
            );
        }

        if primitive_type.is_int() {
            constants.trait_conformance_declarations.add(
                &make_conformance_declaration(&traits.Int, &number_conformance, vec![])
            );
        }
    }

    let and_op = FunctionPointer::make_operator("&&", "and", 2, &bool_type, &bool_type);
    constants.overload_function(&and_op);

    let or__op = FunctionPointer::make_operator("||", "or", 2, &bool_type, &bool_type);
    constants.overload_function(&or__op);

    let not_op = FunctionPointer::make_operator("!", "not", 1, &bool_type, &bool_type);
    constants.overload_function(&not_op);


    Rc::new(Builtins {
        traits,
        operators: Operators {
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
            exponentiate: exp_ops,
            modulo: mod_ops,

            positive: pos_ops,
            negative: neg_ops,
            not: not_op,
        },
        debug: debug::make_functions(&mut constants),
        strings: strings::make(&mut constants),
        precedence_groups,
        primitive_metatypes,
        global_constants: constants,
    })
}
