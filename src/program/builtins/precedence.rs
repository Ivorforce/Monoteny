use std::rc::Rc;
use uuid::Uuid;
use crate::linker::precedence::{OperatorAssociativity, PrecedenceGroup};
use crate::program::functions::ParameterKey;
use crate::program::module::Module;
use crate::program::types::{Pattern, PatternPart};

#[allow(non_snake_case)]
pub struct PrecedenceGroups {
    pub module: Rc<Module>,
    pub list: Vec<Rc<PrecedenceGroup>>,

    pub LeftUnaryPrecedence: Rc<PrecedenceGroup>,
    pub ExponentiationPrecedence: Rc<PrecedenceGroup>,
    pub MultiplicationPrecedence: Rc<PrecedenceGroup>,
    pub AdditionPrecedence: Rc<PrecedenceGroup>,
    pub ComparisonPrecedence: Rc<PrecedenceGroup>,
    pub LogicalConjunctionPrecedence: Rc<PrecedenceGroup>,
    pub LogicalDisjunctionPrecedence: Rc<PrecedenceGroup>,
}

pub fn create() -> PrecedenceGroups {
    let mut module = Module::new("monoteny.precedence".to_string());
    let mut list = vec![];

    let add_precedence_group = |list: &mut Vec<Rc<PrecedenceGroup>>, module: &mut Module, name: &str, associativity: OperatorAssociativity, functions: Vec<(&str, &str)>| -> Rc<PrecedenceGroup> {
        let group = Rc::new(PrecedenceGroup::new(
            name,
            associativity,
        ));
        list.push(Rc::clone(&group));

        for (operator, alias) in functions {
            let operator = String::from(operator);

            module.patterns.insert(Rc::new(Pattern {
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
            }));
        }
        group
    };

    PrecedenceGroups {
        LeftUnaryPrecedence: add_precedence_group(
            &mut list, &mut module, "LeftUnaryPrecedence", OperatorAssociativity::LeftUnary,
            vec![("+", "positive"), ("-", "negative"), ("not", "not_f")]
        ),
        ExponentiationPrecedence: add_precedence_group(
            &mut list, &mut module, "ExponentiationPrecedence", OperatorAssociativity::Right,
            vec![("**", "exponent")]
        ),
        MultiplicationPrecedence: add_precedence_group(
            &mut list, &mut module, "MultiplicationPrecedence", OperatorAssociativity::Left,
            vec![("*", "multiply"), ("/", "divide"), ("%", "modulo")]
        ),
        AdditionPrecedence: add_precedence_group(
            &mut list, &mut module, "AdditionPrecedence", OperatorAssociativity::Left,
            vec![("+", "add"), ("-", "subtract")]
        ),
        ComparisonPrecedence: add_precedence_group(
            &mut list, &mut module, "ComparisonPrecedence", OperatorAssociativity::ConjunctivePairs,
            vec![
                ("==", "is_equal"), ("!=", "is_not_equal"),
                (">", "is_greater"), (">=", "is_greater_or_equal"),
                ("<", "is_lesser"), ("<=", "is_lesser_or_equal")
            ]
        ),
        LogicalConjunctionPrecedence: add_precedence_group(
            &mut list, &mut module, "LogicalConjunctionPrecedence", OperatorAssociativity::Left,
            vec![("and", "and_f")]  // TODO Alias differently?
        ),
        LogicalDisjunctionPrecedence: add_precedence_group(
            &mut list, &mut module, "LogicalDisjunctionPrecedence", OperatorAssociativity::Left,
            vec![("or", "or_f")]
        ),
        module: Rc::new(module),
        list,
    }
}