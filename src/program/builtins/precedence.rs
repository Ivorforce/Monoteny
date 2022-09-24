use std::collections::HashSet;
use std::rc::Rc;
use uuid::Uuid;
use crate::parser;
use crate::parser::associativity::{OperatorAssociativity, PrecedenceGroup};
use crate::program::types::Pattern;

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

pub fn make_groups(parser_scope: &mut parser::scopes::Level) -> TenLangBuiltinPrecedenceGroups {
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

    TenLangBuiltinPrecedenceGroups {
        LeftUnaryPrecedence: add_precedence_group(
            parser_scope, "LeftUnaryPrecedence", OperatorAssociativity::LeftUnary,
            vec![("+", "positive"), ("-", "negative"), ("!", "not")]
        ),
        ExponentiationPrecedence: add_precedence_group(
            parser_scope, "ExponentiationPrecedence", OperatorAssociativity::Right,
            vec![("**", "exponentiate")]
        ),
        MultiplicationPrecedence: add_precedence_group(
            parser_scope, "MultiplicationPrecedence", OperatorAssociativity::Left,
            vec![("*", "multiply"), ("/", "divide"), ("%", "modulo")]
        ),
        AdditionPrecedence: add_precedence_group(
            parser_scope, "AdditionPrecedence", OperatorAssociativity::Left,
            vec![("+", "add"), ("-", "subtract")]
        ),
        ComparisonPrecedence: add_precedence_group(
            parser_scope, "ComparisonPrecedence", OperatorAssociativity::ConjunctivePairs,
            vec![
                ("==", "is_equal"), ("!=", "is_not_equal"),
                (">", "is_greater"), (">=", "is_greater_or_equal"),
                ("<", "is_lesser"), ("<=", "is_lesser_or_equal")
            ]
        ),
        LogicalConjunctionPrecedence: add_precedence_group(
            parser_scope, "LogicalConjunctionPrecedence", OperatorAssociativity::Left,
            vec![("&&", "and")]
        ),
        LogicalDisjunctionPrecedence: add_precedence_group(
            parser_scope, "LogicalDisjunctionPrecedence", OperatorAssociativity::Left,
            vec![("||", "or")]
        ),
    }
}