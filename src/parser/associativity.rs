use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use itertools::Itertools;
use uuid::Uuid;
use crate::linker::link_unary_function;
use crate::parser::abstract_syntax::*;

#[derive(Copy, Clone, PartialEq)]
pub enum BinaryOperatorAssociativity {
    Left,
    Right,
    PairsJoinedByAnds,  // >=, == and the likes
}

pub struct BinaryPrecedenceGroup {
    pub id: Uuid,
    pub name: String,
    pub associativity: BinaryOperatorAssociativity,
}

pub struct BinaryOperatorPattern {
    pub name: String,
    pub precedence_group: Rc<BinaryPrecedenceGroup>,
}

impl BinaryPrecedenceGroup {
    pub fn new(name: &str, associativity: BinaryOperatorAssociativity) -> BinaryPrecedenceGroup {
        BinaryPrecedenceGroup {
            id: Uuid::new_v4(),
            name: String::from(name),
            associativity
        }
    }
}

impl PartialEq for BinaryPrecedenceGroup {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for BinaryPrecedenceGroup {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

pub fn sort_binary_expressions(arguments: Vec<Box<Expression>>, operators: Vec<String>) -> Box<Expression> {
    if arguments.len() != operators.len() + 1 || arguments.len() < 2 {
        panic!("Internal Error for associativity: (args.len(): {}, operators.len(): {})", arguments.len(), operators.len());
    }

    let groups = vec![(
            BinaryPrecedenceGroup::new("ExponentiationPrecedence", BinaryOperatorAssociativity::Right),
            HashSet::from([String::from("**")]),
        ), (
            BinaryPrecedenceGroup::new("MultiplicationPrecedence", BinaryOperatorAssociativity::Left),
            HashSet::from([String::from("*"), String::from("/"), String::from("%")]),
        ), (
            BinaryPrecedenceGroup::new("AdditionPrecedence", BinaryOperatorAssociativity::Left),
            HashSet::from([String::from("+"), String::from("-")]),
        ), (
            BinaryPrecedenceGroup::new("ComparisonPrecedence", BinaryOperatorAssociativity::PairsJoinedByAnds),
            HashSet::from([String::from("=="), String::from("!="), String::from(">"), String::from("<"), String::from(">="), String::from("<=")]),
        ), (
            BinaryPrecedenceGroup::new("LogicalConjunctionPrecedence", BinaryOperatorAssociativity::Left),
            HashSet::from([String::from("&&")]),
        ), (
            BinaryPrecedenceGroup::new("LogicalDisjunctionPrecedence", BinaryOperatorAssociativity::Left),
            HashSet::from([String::from("||")]),
        ),
    ];

    let mut arguments = arguments;
    let mut operators = operators;

    let mut join_binary_at = |arguments: &mut Vec<Box<Expression>>, operators: &mut Vec<String>, i: usize| {
        let lhs = arguments.remove(i);
        let rhs = arguments.remove(i);
        let operator = operators.remove(i);

        arguments.insert(
            i,
            Box::new(Expression::BinaryOperator { lhs, rhs, operator }
        ));
    };

    for (group, group_operators) in groups {
        match group.associativity {
            BinaryOperatorAssociativity::Left => {
                // Iterate left to right
                let mut i = 0;
                while i < operators.len() {
                    if group_operators.contains(&operators[i]) {
                        join_binary_at(&mut arguments, &mut operators, i);
                    }
                    else {
                        i += 1;  // Skip
                    }
                }
            }
            BinaryOperatorAssociativity::Right => {
                // Iterate right to left
                let mut i = operators.len();
                while i > 0 {
                    if group_operators.contains(&operators[i - 1]) {
                        join_binary_at(&mut arguments, &mut operators, i - 1);
                    }
                    else {
                        i -= 1;  // Skip
                    }
                }
            }
            BinaryOperatorAssociativity::PairsJoinedByAnds => {
                // Iteration direction doesn't matter here.
                let mut i = 0;
                while i < operators.len() {
                    if !group_operators.contains(&operators[i]) {
                        // Skip
                        i += 1;
                        continue;
                    }

                    if i + 1 >= operators.len() || !group_operators.contains(&operators[i + 1]) {
                        // Just one operation; let's use a binary operator.
                        join_binary_at(&mut arguments, &mut operators, i);
                        continue;
                    }

                    // More than one operation; Let's build a pairwise operation!
                    // We can start with the first two operators and 3 arguments of which we
                    // know they belong to the operation.
                    let mut group_arguments = vec![
                        arguments.remove(i), arguments.remove(i), arguments.remove(i)
                    ];
                    let mut group_operators = vec![
                        operators.remove(i), operators.remove(i)
                    ];

                    while i < operators.len() && group_operators.contains(&operators[i]) {
                        // Found one more! Yay!
                        group_arguments.push(arguments.remove(i));
                        group_operators.push(operators.remove(i));
                    }

                    // Let's wrap this up.
                    arguments.insert(
                        i,
                        Box::new(Expression::PairAssociativeBinaryOperators {
                            arguments: group_arguments,
                            operators: group_operators
                        }
                    ));
                }
            }
        }

        if operators.len() == 0 {
            // We can return early
            return arguments.into_iter().next().unwrap()
        }
    }

    if operators.len() > 0 {
        panic!("Unrecognized binary operator pattern(s); did you forget an import? Offending Operators: {:?}", &operators);
    }

    arguments.into_iter().next().unwrap()
}