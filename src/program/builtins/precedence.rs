use std::rc::Rc;
use crate::linker::precedence::{OperatorAssociativity, PrecedenceGroup};
use crate::program::module::Module;

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

    let add_precedence_group = |list: &mut Vec<Rc<PrecedenceGroup>>, module: &mut Module, name: &str, associativity: OperatorAssociativity| -> Rc<PrecedenceGroup> {
        let group = Rc::new(PrecedenceGroup::new(
            name,
            associativity,
        ));
        list.push(Rc::clone(&group));
        group
    };

    PrecedenceGroups {
        LeftUnaryPrecedence: add_precedence_group(
            &mut list, &mut module, "LeftUnaryPrecedence", OperatorAssociativity::LeftUnary,
        ),
        ExponentiationPrecedence: add_precedence_group(
            &mut list, &mut module, "ExponentiationPrecedence", OperatorAssociativity::Right,
        ),
        MultiplicationPrecedence: add_precedence_group(
            &mut list, &mut module, "MultiplicationPrecedence", OperatorAssociativity::Left,
        ),
        AdditionPrecedence: add_precedence_group(
            &mut list, &mut module, "AdditionPrecedence", OperatorAssociativity::Left,
        ),
        ComparisonPrecedence: add_precedence_group(
            &mut list, &mut module, "ComparisonPrecedence", OperatorAssociativity::ConjunctivePairs,
        ),
        LogicalConjunctionPrecedence: add_precedence_group(
            &mut list, &mut module, "LogicalConjunctionPrecedence", OperatorAssociativity::Left,
        ),
        LogicalDisjunctionPrecedence: add_precedence_group(
            &mut list, &mut module, "LogicalDisjunctionPrecedence", OperatorAssociativity::Left,
        ),
        module: Rc::new(module),
        list,
    }
}