use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use uuid::Uuid;
use strum::IntoEnumIterator;
use crate::linker::scopes;
use crate::program::traits::{Trait, TraitConformanceDeclaration, TraitConformanceRequirement};
use crate::parser;
use crate::parser::associativity::{OperatorAssociativity, PrecedenceGroup};
use crate::program::types::*;
use crate::program::primitives;
use crate::program;
use crate::program::functions::{FunctionForm, FunctionPointer, HumanFunctionInterface, MachineFunctionInterface};

pub struct TenLangBuiltins {
    pub traits: TenLangBuiltinTraits,
    pub operators: TenLangBuiltinOperators,
    pub functions: TenLangBuiltinFunctions,
    pub primitive_metatypes: HashMap<primitives::Type, Box<Type>>,
    pub structs: TenLangBuiltinStructs,
    pub precedence_groups: TenLangBuiltinPrecedenceGroups,

    pub parser_constants: parser::scopes::Level,
    pub global_constants: scopes::Level,
}

pub struct TenLangBuiltinOperators {
    pub and: Rc<FunctionPointer>,
    pub or: Rc<FunctionPointer>,

    pub equal_to: HashSet<Rc<FunctionPointer>>,
    pub not_equal_to: HashSet<Rc<FunctionPointer>>,

    pub greater_than: HashSet<Rc<FunctionPointer>>,
    pub greater_than_or_equal_to: HashSet<Rc<FunctionPointer>>,
    pub lesser_than: HashSet<Rc<FunctionPointer>>,
    pub lesser_than_or_equal_to: HashSet<Rc<FunctionPointer>>,

    pub add: HashSet<Rc<FunctionPointer>>,
    pub subtract: HashSet<Rc<FunctionPointer>>,
    pub multiply: HashSet<Rc<FunctionPointer>>,
    pub divide: HashSet<Rc<FunctionPointer>>,
    pub exponentiate: HashSet<Rc<FunctionPointer>>,
    pub modulo: HashSet<Rc<FunctionPointer>>,

    pub positive: HashSet<Rc<FunctionPointer>>,
    pub negative: HashSet<Rc<FunctionPointer>>,
    pub not: Rc<FunctionPointer>,
}

pub struct TenLangBuiltinFunctions {
    pub print: Rc<FunctionPointer>,
}

#[allow(non_snake_case)]
pub struct TenLangBuiltinPrecedenceGroups {
    pub LeftUnaryPrecedence: Rc<PrecedenceGroup>,
    pub ExponentiationPrecedence: Rc<PrecedenceGroup>,
    pub MultiplicationPrecedence: Rc<PrecedenceGroup>,
    pub AdditionPrecedence: Rc<PrecedenceGroup>,
    pub ComparisonPrecedence: Rc<PrecedenceGroup>,
    pub LogicalConjunctionPrecedence: Rc<PrecedenceGroup>,
    pub LogicalDisjunctionPrecedence: Rc<PrecedenceGroup>,
}

#[allow(non_snake_case)]
pub struct TenLangBuiltinTraits {
    pub Number: Rc<Trait>,
    pub Float: Rc<Trait>,
    pub Int: Rc<Trait>,
}

#[allow(non_snake_case)]
pub struct TenLangBuiltinStructs {
    pub String: Rc<Struct>,
}

pub struct NumberFunctions {
    pub add: Rc<FunctionPointer>,
    pub subtract: Rc<FunctionPointer>,
    pub multiply: Rc<FunctionPointer>,
    pub divide: Rc<FunctionPointer>,

    pub positive: Rc<FunctionPointer>,
    pub negative: Rc<FunctionPointer>,
}

pub fn make_number_functions(type_: &Box<Type>) -> NumberFunctions {
    NumberFunctions {
        add: FunctionPointer::make_operator("+", "add", 2, type_, type_),
        subtract: FunctionPointer::make_operator("-", "subtract", 2, type_, type_),
        multiply: FunctionPointer::make_operator("*", "multiply", 2, type_, type_),
        divide: FunctionPointer::make_operator("/", "divide", 2, type_, type_),

        positive: FunctionPointer::make_operator("+", "is_lesser_or_equal", 1, type_, type_),
        negative: FunctionPointer::make_operator("-", "is_lesser_or_equal", 1, type_, type_),
    }
}

pub fn create_builtins() -> Rc<TenLangBuiltins> {
    let mut constants: scopes::Level = scopes::Level::new();

    let bool_type = Type::unit(TypeUnit::Primitive(primitives::Type::Bool));
    let generic_id = Uuid::new_v4();
    let generic_type = Type::unit(TypeUnit::Any(generic_id));

    let primitive_metatypes = primitives::Type::iter()
        .map(|x| (x, Box::new(Type {
            unit: TypeUnit::MetaType,
            arguments: vec![Box::new(Type {
                unit: TypeUnit::Primitive(x),
                arguments: vec![]
            })],
        })))
        .collect::<HashMap<primitives::Type, Box<Type>>>();

    for (primitive_type, metatype) in &primitive_metatypes {
        constants.insert_singleton(
            scopes::Environment::Global,
            Variable::make_immutable(metatype.clone()),
            &primitive_type.identifier_string()
        );
    }

    let add_struct = |constants: &mut scopes::Level, name: &str| -> Rc<Struct> {
        let name = String::from(name);

        let s = Rc::new(Struct {
            id: Uuid::new_v4(),
            name: name.clone(),
        });
        let s_type = Box::new(Type {
            unit: TypeUnit::MetaType,
            arguments: vec![Type::unit(TypeUnit::Struct(Rc::clone(&s)))]
        });

        constants.insert_singleton(
            scopes::Environment::Global,
            Variable::make_immutable(s_type),
            &name
        );

        s
    };


    let add_precedence_group = |scope: &mut parser::scopes::Level, name: &str, associativity: OperatorAssociativity, operators: Vec<(&str, &str)>| -> Rc<PrecedenceGroup> {
        let group = Rc::new(PrecedenceGroup::new(name, associativity));
        scope.precedence_groups.push((Rc::clone(&group), HashSet::new()));

        for (operator, alias) in operators {
            scope.add_pattern(Pattern {
                id: Uuid::new_v4(),
                operator: String::from(operator),
                alias: String::from(alias),
                precedence_group: Rc::clone(&group),
            });
        }
        group
    };


    let mut parser_scope = parser::scopes::Level::new();

    let precedence_groups = TenLangBuiltinPrecedenceGroups {
        LeftUnaryPrecedence: add_precedence_group(
            &mut parser_scope, "LeftUnaryPrecedence", OperatorAssociativity::LeftUnary,
            vec![("+", "positive"), ("-", "negative"), ("!", "not")]
        ),
        ExponentiationPrecedence: add_precedence_group(
            &mut parser_scope, "ExponentiationPrecedence", OperatorAssociativity::Right,
            vec![("**", "exponentiate")]
        ),
        MultiplicationPrecedence: add_precedence_group(
            &mut parser_scope, "MultiplicationPrecedence", OperatorAssociativity::Left,
            vec![("*", "multiply"), ("/", "divide"), ("%", "modulo")]
        ),
        AdditionPrecedence: add_precedence_group(
            &mut parser_scope, "AdditionPrecedence", OperatorAssociativity::Left,
            vec![("+", "add"), ("-", "subtract")]
        ),
        ComparisonPrecedence: add_precedence_group(
            &mut parser_scope, "ComparisonPrecedence", OperatorAssociativity::ConjunctivePairs,
            vec![
                ("==", "is_equal"), ("!=", "is_not_equal"),
                (">", "is_greater"), (">=", "is_greater_or_equal"),
                ("<", "is_lesser"), ("<=", "is_lesser_or_equal")
            ]
        ),
        LogicalConjunctionPrecedence: add_precedence_group(
            &mut parser_scope, "LogicalConjunctionPrecedence", OperatorAssociativity::Left,
            vec![("&&", "and")]
        ),
        LogicalDisjunctionPrecedence: add_precedence_group(
            &mut parser_scope, "LogicalDisjunctionPrecedence", OperatorAssociativity::Left,
            vec![("||", "or")]
        ),
    };


    let make_trait = |name: &str, generic_id: &Uuid, fns: Vec<&Rc<FunctionPointer>>, parents: Vec<Rc<Trait>>| -> Rc<Trait> {
        let generic_type = Type::unit(TypeUnit::Any(*generic_id));

        let mut t = Trait {
            id: Uuid::new_v4(),
            name: String::from(name),
            parameters: vec![*generic_id],
            abstract_functions: fns.into_iter().map(Rc::clone).collect(),
            requirements: HashSet::new(),
        };

        for parent in parents {
            TraitConformanceRequirement::bind(&parent, vec![generic_type.clone()], &mut t.requirements);
        }

        return Rc::new(t)
    };


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


    let abstract_number_functions = make_number_functions(&generic_type);
    add_ops.insert(Rc::clone(&abstract_number_functions.add));
    sub_ops.insert(Rc::clone(&abstract_number_functions.subtract));
    mul_ops.insert(Rc::clone(&abstract_number_functions.multiply));
    div_ops.insert(Rc::clone(&abstract_number_functions.divide));
    pos_ops.insert(Rc::clone(&abstract_number_functions.positive));
    neg_ops.insert(Rc::clone(&abstract_number_functions.negative));

    let number_trait = make_trait("Number", &generic_id, vec![
        &abstract_number_functions.add,
        &abstract_number_functions.subtract,
        &abstract_number_functions.multiply,
        &abstract_number_functions.divide,
        &abstract_number_functions.positive,
        &abstract_number_functions.negative
    ], vec![]);
    constants.add_trait(&number_trait);

    let float_trait = make_trait("Float", &generic_id, vec![], vec![Rc::clone(&number_trait)]);
    constants.add_trait(&float_trait);

    let int_trait = make_trait("Int", &generic_id, vec![], vec![Rc::clone(&number_trait)]);
    constants.add_trait(&int_trait);

    let traits = TenLangBuiltinTraits {
        Number: number_trait,
        Float: float_trait,
        Int: int_trait,
    };


    for primitive_type in primitives::Type::iter() {
        let type_ = &Type::unit(TypeUnit::Primitive(primitive_type));

        // Pair-Associative
        let eq__op = FunctionPointer::make_operator("==", "is_equal", 2, type_, &bool_type);
        constants.add_function(&eq__op);
        eq__ops.insert(eq__op);

        let neq_op = FunctionPointer::make_operator("!=", "is_not_equal", 2, type_, &bool_type);
        constants.add_function(&neq_op);
        neq_ops.insert(neq_op);

        if !primitive_type.is_number() {
            continue;
        }

        let number_functions = make_number_functions(&type_);
        add_ops.insert(Rc::clone(&number_functions.add));
        constants.add_function(&number_functions.add);

        sub_ops.insert(Rc::clone(&number_functions.subtract));
        constants.add_function(&number_functions.subtract);

        mul_ops.insert(Rc::clone(&number_functions.multiply));
        constants.add_function(&number_functions.multiply);

        div_ops.insert(Rc::clone(&number_functions.divide));
        constants.add_function(&number_functions.divide);

        pos_ops.insert(Rc::clone(&number_functions.positive));
        constants.add_function(&number_functions.positive);

        neg_ops.insert(Rc::clone(&number_functions.negative));
        constants.add_function(&number_functions.negative);

        let mod_op = FunctionPointer::make_operator("%", "modulo", 2, type_, type_);
        constants.add_function(&mod_op);
        mod_ops.insert(Rc::clone(&mod_op));

        let number_conformance = Rc::new(TraitConformanceDeclaration {
            id: Uuid::new_v4(),
            trait_: Rc::clone(&traits.Number),
            arguments: vec![type_.clone()],
            requirements: HashSet::new(),
            trait_requirements_conformance: HashMap::new(),
            function_implementations: HashMap::from([
                (Rc::clone(&abstract_number_functions.add), Rc::clone(&abstract_number_functions.add)),
                (Rc::clone(&abstract_number_functions.subtract), Rc::clone(&abstract_number_functions.subtract)),
                (Rc::clone(&abstract_number_functions.multiply), Rc::clone(&abstract_number_functions.multiply)),
                (Rc::clone(&abstract_number_functions.divide), Rc::clone(&abstract_number_functions.divide)),

                (Rc::clone(&abstract_number_functions.positive), Rc::clone(&abstract_number_functions.positive)),
                (Rc::clone(&abstract_number_functions.negative), Rc::clone(&abstract_number_functions.negative)),
            ])
        });
        constants.trait_conformance_declarations.add(Rc::clone(&number_conformance));

        // Pair-Associative
        let gr__op = FunctionPointer::make_operator(">", "is_greater", 2, type_, &bool_type);
        constants.add_function(&gr__op);
        gr__ops.insert(gr__op);

        let geq_op = FunctionPointer::make_operator(">=", "is_greater_or_equal", 2, type_, &bool_type);
        constants.add_function(&geq_op);
        geq_ops.insert(geq_op);

        let le__op = FunctionPointer::make_operator("<", "is_lesser", 2, type_, &bool_type);
        constants.add_function(&le__op);
        le__ops.insert(le__op);

        let leq_op = FunctionPointer::make_operator("<=", "is_lesser_or_equal", 2, type_, &bool_type);
        constants.add_function(&leq_op);
        leq_ops.insert(leq_op);

        if primitive_type.is_float() {
            let exp_op = FunctionPointer::make_operator("**", "exponentiate", 2, type_, type_);
            constants.add_function(&exp_op);
            exp_ops.insert(Rc::clone(&exp_op));

            constants.trait_conformance_declarations.add(
                TraitConformanceDeclaration::create_for_trivial_inheritance(&traits.Float, &number_conformance)
            );
        }

        if primitive_type.is_int() {
            constants.trait_conformance_declarations.add(
                TraitConformanceDeclaration::create_for_trivial_inheritance(&traits.Int, &number_conformance)
            );
        }
    }

    let and_op = FunctionPointer::make_operator("&&", "and", 2, &bool_type, &bool_type);
    constants.add_function(&and_op);

    let or__op = FunctionPointer::make_operator("||", "or", 2, &bool_type, &bool_type);
    constants.add_function(&or__op);

    let not_op = FunctionPointer::make_operator("!", "not", 1, &bool_type, &bool_type);
    constants.add_function(&not_op);


    let print_function = FunctionPointer::make_global("print", "print", [generic_type.clone()].into_iter(), None);
    constants.add_function(&print_function);

    Rc::new(TenLangBuiltins {
        traits,
        operators: TenLangBuiltinOperators {
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
        functions: TenLangBuiltinFunctions {
            print: print_function,
        },
        structs: TenLangBuiltinStructs {
            String: add_struct(&mut constants, "String")
        },
        precedence_groups,
        primitive_metatypes,
        parser_constants: parser_scope,
        global_constants: constants,
    })
}
