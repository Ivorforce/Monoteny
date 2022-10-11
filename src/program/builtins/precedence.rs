use std::collections::HashSet;
use std::rc::Rc;
use uuid::Uuid;
use crate::parser;
use crate::linker::precedence::{OperatorAssociativity, PrecedenceGroup};
use crate::linker::scopes;
use crate::parser::abstract_syntax::PatternForm;
use crate::program::types::Pattern;

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
    let add_precedence_group = |scope: &mut scopes::Scope, name: &str, associativity: OperatorAssociativity, operators: Vec<(&str, &str)>| -> Rc<PrecedenceGroup> {
        let group = Rc::new(PrecedenceGroup::new(
            name,
            associativity,
            if associativity == OperatorAssociativity::LeftUnary { PatternForm::Unary } else { PatternForm::Binary }
        ));
        scope.precedence_groups.push((Rc::clone(&group), HashSet::new()));

        for (operator, alias) in operators {
            scope.add_pattern(Rc::new(Pattern {
                id: Uuid::new_v4(),
                operator: String::from(operator),
                alias: String::from(alias),
                precedence_group: Rc::clone(&group),
            }));
        }
        group
    };

    PrecedenceGroups {
        LeftUnaryPrecedence: add_precedence_group(
            scope, "LeftUnaryPrecedence", OperatorAssociativity::LeftUnary,
            vec![("+", "positive"), ("-", "negative"), ("!", "not")]
        ),
        ExponentiationPrecedence: add_precedence_group(
            scope, "ExponentiationPrecedence", OperatorAssociativity::Right,
            vec![("**", "exponentiate")]
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
            vec![("&&", "and")]
        ),
        LogicalDisjunctionPrecedence: add_precedence_group(
            scope, "LogicalDisjunctionPrecedence", OperatorAssociativity::Left,
            vec![("||", "or")]
        ),
    }
}