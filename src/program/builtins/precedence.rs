use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use uuid::Uuid;
use crate::parser;
use crate::linker::precedence::{OperatorAssociativity, PrecedenceGroup};
use crate::linker::scopes;
use crate::program::functions::ParameterKey;
use crate::program::types::{Pattern, PatternPart};

#[allow(non_snake_case)]
pub struct PrecedenceGroups {
    pub LeftUnaryPrecedence: Rc<PrecedenceGroup>,
    pub ExponentiationPrecedence: Rc<PrecedenceGroup>,
    pub MultiplicationPrecedence: Rc<PrecedenceGroup>,
    pub AdditionPrecedence: Rc<PrecedenceGroup>,
    pub ComparisonPrecedence: Rc<PrecedenceGroup>,
    pub LogicalConjunctionPrecedence: Rc<PrecedenceGroup>,
    pub LogicalDisjunctionPrecedence: Rc<PrecedenceGroup>,
}

pub fn make_groups(scope: &mut scopes::Scope) -> PrecedenceGroups {
    let add_precedence_group = |scope: &mut scopes::Scope, name: &str, associativity: OperatorAssociativity, functions: Vec<(&str, &str)>| -> Rc<PrecedenceGroup> {
        let group = Rc::new(PrecedenceGroup::new(
            name,
            associativity,
        ));
        scope.precedence_groups.push((Rc::clone(&group), HashMap::new()));

        for (operator, alias) in functions {
            let operator = String::from(operator);

            scope.add_pattern(Rc::new(Pattern {
                id: Uuid::new_v4(),
                alias: String::from(alias),
                precedence_group: Rc::clone(&group),
                parts: if associativity == OperatorAssociativity::LeftUnary {
                    vec![
                        Box::new(PatternPart::Keyword(operator)),
                        Box::new(PatternPart::Parameter { key: ParameterKey::Positional, internal_name: String::from("arg") }),
                    ]
                } else {
                    vec![
                        Box::new(PatternPart::Parameter { key: ParameterKey::Positional, internal_name: String::from("lhs") }),
                        Box::new(PatternPart::Keyword(operator)),
                        Box::new(PatternPart::Parameter { key: ParameterKey::Positional, internal_name: String::from("rhs") }),
                    ]
                },
            })).unwrap();
        }
        group
    };

    PrecedenceGroups {
        LeftUnaryPrecedence: add_precedence_group(
            scope, "LeftUnaryPrecedence", OperatorAssociativity::LeftUnary,
            vec![("+", "positive"), ("-", "negative"), ("not", "not_f")]
        ),
        ExponentiationPrecedence: add_precedence_group(
            scope, "ExponentiationPrecedence", OperatorAssociativity::Right,
            vec![("**", "exponent")]
        ),
        MultiplicationPrecedence: add_precedence_group(
            scope, "MultiplicationPrecedence", OperatorAssociativity::Left,
            vec![("*", "multiply"), ("/", "divide"), ("%", "modulo")]
        ),
        AdditionPrecedence: add_precedence_group(
            scope, "AdditionPrecedence", OperatorAssociativity::Left,
            vec![("+", "add"), ("-", "subtract")]
        ),
        ComparisonPrecedence: add_precedence_group(
            scope, "ComparisonPrecedence", OperatorAssociativity::ConjunctivePairs,
            vec![
                ("==", "is_equal"), ("!=", "is_not_equal"),
                (">", "is_greater"), (">=", "is_greater_or_equal"),
                ("<", "is_lesser"), ("<=", "is_lesser_or_equal")
            ]
        ),
        LogicalConjunctionPrecedence: add_precedence_group(
            scope, "LogicalConjunctionPrecedence", OperatorAssociativity::Left,
            vec![("and", "and_f")]  // TODO Alias differently?
        ),
        LogicalDisjunctionPrecedence: add_precedence_group(
            scope, "LogicalDisjunctionPrecedence", OperatorAssociativity::Left,
            vec![("or", "or_f")]
        ),
    }
}