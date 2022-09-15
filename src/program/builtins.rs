use std::collections::{HashMap, HashSet};
use std::iter::zip;
use std::rc::Rc;
use uuid::Uuid;
use strum::IntoEnumIterator;
use crate::linker::scopes;
use crate::program::traits::{Trait, TraitConformanceDeclaration};
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

    pub equal_to: Vec<Rc<FunctionPointer>>,
    pub not_equal_to: Vec<Rc<FunctionPointer>>,

    pub greater_than: Vec<Rc<FunctionPointer>>,
    pub greater_than_or_equal_to: Vec<Rc<FunctionPointer>>,
    pub lesser_than: Vec<Rc<FunctionPointer>>,
    pub lesser_than_or_equal_to: Vec<Rc<FunctionPointer>>,

    pub add: Vec<Rc<FunctionPointer>>,
    pub subtract: Vec<Rc<FunctionPointer>>,
    pub multiply: Vec<Rc<FunctionPointer>>,
    pub divide: Vec<Rc<FunctionPointer>>,
    pub exponentiate: Vec<Rc<FunctionPointer>>,
    pub modulo: Vec<Rc<FunctionPointer>>,

    pub positive: Vec<Rc<FunctionPointer>>,
    pub negative: Vec<Rc<FunctionPointer>>,
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
}

#[allow(non_snake_case)]
pub struct TenLangBuiltinStructs {
    pub String: Rc<Struct>,
}

pub fn create_builtins() -> Rc<TenLangBuiltins> {
    let mut constants: scopes::Level = scopes::Level::new();

    let bool_type = Type::unit(TypeUnit::Primitive(primitives::Type::Bool));
    let generic_type = Type::make_any();

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
        constants.insert_singleton(scopes::Environment::Global, Rc::new(Variable {
            id: Uuid::new_v4(),
            type_declaration: metatype.clone(),
            mutability: Mutability::Immutable
        }), &primitive_type.identifier_string());
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

        constants.insert_singleton(scopes::Environment::Global, Rc::new(Variable {
            id: Uuid::new_v4(),
            type_declaration: s_type,
            mutability: Mutability::Immutable,
        }), &name);

        s
    };


    let all_primitives: Vec<Box<Type>> = primitives::Type::iter()
        .map(|x| Type::unit(TypeUnit::Primitive(x)))
        .collect();
    let number_primitives: Vec<Box<Type>> = primitives::Type::NUMBERS.iter()
        .map(|x| Type::unit(TypeUnit::Primitive(*x)))
        .collect();

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


    let add_trait_with_xx_x = |constants: &mut scopes::Level, name: &str, fns: Vec<(&str, &str)>| -> (Rc<Trait>, Vec<Rc<FunctionPointer>>) {
        let generic_id = Uuid::new_v4();
        let generic_type = Type::unit(TypeUnit::Any(generic_id));

        let mut t = Trait {
            id: Uuid::new_v4(),
            name: String::from(name),
            abstract_functions: HashSet::new()
        };

        let mut functions = vec![];
        for (fn_name, fn_alphanumeric_name) in fns {
            // Abstract functions are only added to scope on trait requirements.
            let fun = FunctionPointer::make_operator(fn_name, fn_alphanumeric_name, 2, &generic_type, &generic_type);
            t.abstract_functions.insert(Rc::clone(&fun));
            functions.push(fun);
        }

        let t = Rc::new(t);
        constants.insert_singleton(
            scopes::Environment::Global,
            Rc::new(Variable {
                id: Uuid::new_v4(),
                type_declaration: Type::unit(TypeUnit::Trait(t.clone())),
                mutability: Mutability::Immutable,
            }),
            &t.name
        );
        return (t, functions)
    };

    let (number_trait, number_fns) = add_trait_with_xx_x(&mut constants, "Number", vec![("+", "add"), ("-", "subtract"), ("*", "multiply"), ("/", "divide")]);

    let traits = TenLangBuiltinTraits {
        Number: number_trait
    };

    let mut add_ops: Vec<Rc<FunctionPointer>> = vec![Rc::clone(&number_fns[0])];
    let mut sub_ops: Vec<Rc<FunctionPointer>> = vec![Rc::clone(&number_fns[1])];
    let mut mul_ops: Vec<Rc<FunctionPointer>> = vec![Rc::clone(&number_fns[2])];
    let mut div_ops: Vec<Rc<FunctionPointer>> = vec![Rc::clone(&number_fns[3])];

    let mut exp_ops: Vec<Rc<FunctionPointer>> = vec![];
    let mut mod_ops: Vec<Rc<FunctionPointer>> = vec![];

    let mut gr__ops: Vec<Rc<FunctionPointer>> = vec![];
    let mut geq_ops: Vec<Rc<FunctionPointer>> = vec![];
    let mut le__ops: Vec<Rc<FunctionPointer>> = vec![];
    let mut leq_ops: Vec<Rc<FunctionPointer>> = vec![];

    let mut pos_ops: Vec<Rc<FunctionPointer>> = vec![];
    let mut neg_ops: Vec<Rc<FunctionPointer>> = vec![];

    for primitive_type in number_primitives.iter() {
        let add_op = FunctionPointer::make_operator("+", "add", 2, primitive_type, primitive_type);
        constants.add_function(Rc::clone(&add_op));
        add_ops.push(Rc::clone(&add_op));

        let sub_op = FunctionPointer::make_operator("-", "subtract", 2, primitive_type, primitive_type);
        constants.add_function(Rc::clone(&sub_op));
        sub_ops.push(Rc::clone(&sub_op));

        let mul_op = FunctionPointer::make_operator("*", "multiply",  2, primitive_type, primitive_type);
        constants.add_function(Rc::clone(&mul_op));
        mul_ops.push(Rc::clone(&mul_op));

        let div_op = FunctionPointer::make_operator("/", "divide", 2, primitive_type, primitive_type);
        constants.add_function(Rc::clone(&div_op));
        div_ops.push(Rc::clone(&div_op));

        // TODO Exponentiate should work only on floats and uints, I think
        let exp_op = FunctionPointer::make_operator("**", "exponentiate", 2, primitive_type, primitive_type);
        constants.add_function(Rc::clone(&exp_op));
        exp_ops.push(Rc::clone(&exp_op));

        let mod_op = FunctionPointer::make_operator("%", "modulo", 2, primitive_type, primitive_type);
        constants.add_function(Rc::clone(&mod_op));
        mod_ops.push(Rc::clone(&mod_op));

        constants.trait_conformance_declarations.add(Rc::new(TraitConformanceDeclaration {
            id: Uuid::new_v4(),
            trait_: Rc::clone(&traits.Number),
            arguments: vec![primitive_type.clone()],
            requirements: HashSet::new(),
            function_implementations: HashMap::from([
                (Rc::clone(&number_fns[0]), add_op),
                (Rc::clone(&number_fns[1]), sub_op),
                (Rc::clone(&number_fns[2]), mul_op),
                (Rc::clone(&number_fns[3]), div_op),
            ])
        }));

        // Pair-Associative
        let gr__op = FunctionPointer::make_operator(">", "is_greater", 2, primitive_type, &bool_type);
        constants.add_function(Rc::clone(&gr__op));
        gr__ops.push(gr__op);

        let geq_op = FunctionPointer::make_operator(">=", "is_greater_or_equal", 2, primitive_type, &bool_type);
        constants.add_function(Rc::clone(&geq_op));
        geq_ops.push(geq_op);

        let le__op = FunctionPointer::make_operator("<", "is_lesser", 2, primitive_type, &bool_type);
        constants.add_function(Rc::clone(&le__op));
        le__ops.push(le__op);

        let leq_op = FunctionPointer::make_operator("<=", "is_lesser_or_equal", 2, primitive_type, &bool_type);
        constants.add_function(Rc::clone(&leq_op));
        leq_ops.push(leq_op);

        // Unary + -
        let pos_op = FunctionPointer::make_operator("+", "is_lesser_or_equal", 1, primitive_type, primitive_type);
        constants.add_function(Rc::clone(&pos_op));
        pos_ops.push(pos_op);

        let neg_op = FunctionPointer::make_operator("-", "is_lesser_or_equal", 1, primitive_type, primitive_type);
        constants.add_function(Rc::clone(&neg_op));
        neg_ops.push(neg_op);
    }

    let mut eq__ops: Vec<Rc<FunctionPointer>> = vec![];
    let mut neq_ops: Vec<Rc<FunctionPointer>> = vec![];
    for primitive_type in all_primitives.iter() {
        // Pair-Associative
        let eq__op = FunctionPointer::make_operator("==", "is_equal", 2, primitive_type, &bool_type);
        constants.add_function(Rc::clone(&eq__op));
        eq__ops.push(eq__op);

        let neq_op = FunctionPointer::make_operator("!=", "is_not_equal", 2, primitive_type, &bool_type);
        constants.add_function(Rc::clone(&neq_op));
        neq_ops.push(neq_op);
    }

    let and_op = FunctionPointer::make_operator("&&", "and", 2, &bool_type, &bool_type);
    constants.add_function(Rc::clone(&and_op));

    let or__op = FunctionPointer::make_operator("||", "or", 2, &bool_type, &bool_type);
    constants.add_function(Rc::clone(&or__op));

    let not_op = FunctionPointer::make_operator("!", "not", 1, &bool_type, &bool_type);
    constants.add_function(Rc::clone(&not_op));


    let print_function = FunctionPointer::make_global("print", "print", [generic_type.clone()].into_iter(), None);
    constants.add_function(Rc::clone(&print_function));

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
