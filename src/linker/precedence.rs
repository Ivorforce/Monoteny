use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use uuid::Uuid;
use crate::linker::imperative::ImperativeLinker;
use crate::linker::{scopes, LinkError};
use crate::parser::abstract_syntax::*;
use crate::program::computation_tree::{ExpressionID, ExpressionOperation};
use crate::program::functions::{FunctionOverload, FunctionPointer, ParameterKey};
use crate::program::primitives;
use crate::program::types::{TypeProto, TypeUnit};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum OperatorAssociativity {
    LeftUnary,  // Evaluated with the operator left of the expression.
    Left,  // Left evaluated first.
    Right, // Right evaluated first.
    None,  // Fail parsing if more than one neighboring operator is found.
    ConjunctivePairs, // Evaluated in pairs, joined by && operations.
}

#[derive(Eq)]
pub struct PrecedenceGroup {
    pub id: Uuid,
    pub name: String,
    pub associativity: OperatorAssociativity,
    pub form: PatternForm,
}

#[derive(Clone, PartialEq, Eq)]
pub struct OperatorPattern {
    pub name: String,
    pub precedence_group: Rc<PrecedenceGroup>,
}

impl PrecedenceGroup {
    pub fn new(name: &str, associativity: OperatorAssociativity, form: PatternForm) -> PrecedenceGroup {
        PrecedenceGroup {
            id: Uuid::new_v4(),
            name: String::from(name),
            associativity,
            form
        }
    }
}

impl PartialEq for PrecedenceGroup {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for PrecedenceGroup {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

#[derive(Clone)]
pub enum Token {
    Operator(Rc<FunctionOverload>),
    Expression(ExpressionID),
    AnonymousStruct { keys: Vec<ParameterKey>, values: Vec<ExpressionID> },
    AnonymousArray { keys: Vec<Option<ExpressionID>>, values: Vec<ExpressionID> },
    FunctionReference { overload: Rc<FunctionOverload>, target: Option<ExpressionID> },
}

pub fn link_patterns(mut tokens: Vec<Token>, scope: &scopes::Scope, linker: &mut ImperativeLinker) -> Result<ExpressionID, LinkError> {
    // Resolve structs and array literals
    let mut i = 0;
    for _ in 0 .. tokens.len() {
        match &tokens[i] {
            Token::AnonymousStruct { keys, values } => {
                match if i > 0 { tokens.get(i - 1) } else { None } {
                    Some(Token::FunctionReference { overload, target }) => {
                        tokens[i] = Token::Expression(linker.link_function_call(
                            &overload.pointers,
                            &overload.name,
                            target.iter().map(|_| &ParameterKey::Positional).chain(keys).map(Clone::clone).collect(),
                            target.iter().chain(values).map(Clone::clone).collect(),
                            scope
                        )?);
                        tokens.remove(i - 1);
                    }
                    Some(Token::Expression(expression)) => {
                        return Err(LinkError::LinkError { msg: String::from("Object calls are not yet supported.") })
                    }
                    _ => {
                        if values.len() == 1 && keys.iter().next().unwrap() == &ParameterKey::Positional {
                            tokens[i] = Token::Expression(*values.iter().next().unwrap());
                        }
                        else {
                            return Err(LinkError::LinkError { msg: String ::from("Anonymous struct literals are not yet supported.") })

                        }
                    }
                }
            }
            Token::AnonymousArray { keys, values } => {
                match if i > 0 { tokens.get(i - 1) } else { None } {
                    Some(Token::FunctionReference { overload, target }) => {
                        return Err(LinkError::LinkError { msg: String::from("Functions with subscript form are not yet supported.") })
                    }
                    Some(Token::Expression(expression)) => {
                        return Err(LinkError::LinkError { msg: String::from("Object subscript is not yet supported.") })
                    }
                    _ => {
                        return Err(LinkError::LinkError { msg: String::from("Array literals are not yet supported.") })

                        // let supertype = linker.expressions.type_forest.merge_all(values)?.clone();
                        //
                        // tokens[i] = Token::Expression(linker.link_unambiguous_expression(
                        //     vec![],
                        //     &TypeProto::monad(TypeProto::unit(TypeUnit::Generic(supertype))),
                        //     ExpressionOperation::ArrayLiteral
                        // )?)
                    }
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    // Resolve function references as curried functions
    for i in 0 .. tokens.len() {
        if let Token::FunctionReference { overload, target } = &tokens[i] {
            Err(LinkError::LinkError { msg: String::from("Function references are not yet supported.") })?;
        }
    }

    let mut arguments: Vec<ExpressionID> = vec![];
    let mut operators: Vec<Rc<FunctionOverload>> = vec![];

    if let Token::Expression(expression) = tokens.remove(tokens.len() - 1) {
        arguments.push(expression);
    }
    else {
        return Err(LinkError::LinkError { msg: String::from("Expression missing the final argument.") })
    }

    // Reduce all unary operators, and build interspersed arguments / operators list.
    while !tokens.is_empty() {
        guard!(let Token::Operator(operator) = tokens.remove(tokens.len() - 1) else {
            return Err(LinkError::LinkError { msg: String::from("Expecting an operator but got an expression.") })
        });

        if !tokens.is_empty() {
            if let Token::Expression(expression) = &tokens[tokens.len() - 1] {
                // Binary Operator, because left of operator is an expression!
                arguments.insert(0, expression.clone());
                operators.insert(0, operator);
                tokens.remove(tokens.len() - 1);

                continue
            }
        }

        // Unary operator, because left of operator is an operator!
        let argument = arguments.remove(0);
        arguments.insert(0, linker.link_unary_function(&operator, argument, scope)?);
    }

    if arguments.len() == 1 {
        return Ok(arguments.remove(0))
    }

    // Resolve binary operators. At this point, we have only expressions interspersed with operators.

    let join_binary_at = |linker: &mut ImperativeLinker, arguments: &mut Vec<ExpressionID>, operators: &mut Vec<Rc<FunctionOverload>>, i: usize| -> Result<(), LinkError> {
        let lhs = arguments.remove(i);
        let rhs = arguments.remove(i);
        let operator = operators.remove(i);

        Ok(arguments.insert(
            i,
            linker.link_binary_function(lhs, &operator, rhs, scope)?
        ))
    };

    for (group, group_operators) in &scope.precedence_groups {
        match group.associativity {
            OperatorAssociativity::Left => {
                // Iterate left to right
                let mut i = 0;
                while i < operators.len() {
                    if group_operators.contains(&operators[i].name) {
                        join_binary_at(linker, &mut arguments, &mut operators, i)?;
                    }
                    else {
                        i += 1;  // Skip
                    }
                }
            }
            OperatorAssociativity::Right => {
                // Iterate right to left
                let mut i = operators.len();
                while i > 0 {
                    i -= 1;
                    if group_operators.contains(&operators[i].name) {
                        join_binary_at(linker, &mut arguments, &mut operators, i)?;
                    }
                }
            }
            OperatorAssociativity::None => {
                // Iteration direction doesn't matter here.
                let mut i = 0;
                while i < operators.len() {
                    if group_operators.contains(&operators[i].name) {
                        if i + 1 < group_operators.len() && group_operators.contains(&operators[i + 1].name) {
                            panic!("Cannot parse two neighboring {} operators because no associativity is defined.", &operators[i].name);
                        }

                        join_binary_at(linker, &mut arguments, &mut operators, i)?;
                    }

                    i += 1;
                }
            }
            OperatorAssociativity::ConjunctivePairs => {
                // Iteration direction doesn't matter here.
                let mut i = 0;
                while i < operators.len() {
                    if !group_operators.contains(&operators[i].name) {
                        // Skip
                        i += 1;
                        continue;
                    }

                    if i + 1 >= operators.len() || !group_operators.contains(&operators[i + 1].name) {
                        // Just one operation; let's use a binary operator.
                        join_binary_at(linker, &mut arguments, &mut operators, i)?;
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
                    arguments.insert(i, linker.link_conjunctive_pairs(
                        group_arguments,
                        group_operators
                    )?);
                }
            }
            // Unary operators are already resolved at this stage.
            OperatorAssociativity::LeftUnary => {}
        }

        if operators.len() == 0 {
            // We can return early
            return Ok(arguments.into_iter().next().unwrap())
        }
    }

    if operators.len() > 0 {
        panic!("Unrecognized binary operator pattern(s); did you forget an import? Offending Operators: {:?}", &operators.iter().map(|x| &x.name).collect_vec());
    }

    Ok(arguments.into_iter().next().unwrap())
}
