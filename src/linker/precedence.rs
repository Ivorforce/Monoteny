use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use uuid::Uuid;
use crate::linker::imperative::ImperativeLinker;
use crate::linker::{scopes, LinkError};
use crate::linker::scopes::Environment;
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
}

#[derive(Clone, PartialEq, Eq)]
pub struct OperatorPattern {
    pub name: String,
    pub precedence_group: Rc<PrecedenceGroup>,
}

impl PrecedenceGroup {
    pub fn new(name: &str, associativity: OperatorAssociativity) -> PrecedenceGroup {
        PrecedenceGroup {
            id: Uuid::new_v4(),
            name: String::from(name),
            associativity,
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
    Keyword(String),
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
                            &overload.functions(),
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
            match overload.pointers.len() {
                1 => {
                    let ref_ = overload.pointers.iter().next().unwrap();
                    let function = ref_.as_function_pointer()?;

                    tokens[i] = Token::Expression(linker.link_unambiguous_expression(
                        vec![],
                        &TypeProto::unit(TypeUnit::Function(Rc::clone(function))),
                        ExpressionOperation::VariableLookup(ref_.clone())
                    )?);
                }
                _ => Err(LinkError::LinkError {
                    msg: String::from("References to overloaded functions are not yet supported (need syntax to distinguish which to choose).")
                })?,
            }
        }
    }

    let mut arguments: Vec<ExpressionID> = vec![];
    let mut keywords: Vec<String> = vec![];

    if let Token::Expression(expression) = tokens.remove(tokens.len() - 1) {
        arguments.push(expression);
    }
    else {
        return Err(LinkError::LinkError { msg: String::from("Expression missing the final argument.") })
    }

    // Reduce all unary operators, and build interspersed arguments / operators list.
    let left_unary_operators = &scope.precedence_groups[0].1;
    while !tokens.is_empty() {
        guard!(let Token::Keyword(keyword) = tokens.remove(tokens.len() - 1) else {
            return Err(LinkError::LinkError { msg: String::from("Expecting an operator but got an expression.") })
        });

        if !tokens.is_empty() {
            if let Token::Expression(expression) = &tokens[tokens.len() - 1] {
                // Binary Operator, because left of operator is an expression!
                arguments.insert(0, expression.clone());
                keywords.insert(0, keyword);
                tokens.remove(tokens.len() - 1);

                continue
            }
        }

        let overload = scope.resolve(Environment::Global, &left_unary_operators[&keyword])?.as_function_overload()?;

        // Unary operator, because left of operator is an operator!
        let argument = arguments.remove(0);
        arguments.insert(0, linker.link_function_call(&overload.functions(), &overload.name, vec![ParameterKey::Positional], vec![argument], scope)?);
    }

    if arguments.len() == 1 {
        // Just one argument, we can shortcut!
        return Ok(arguments.remove(0))
    }

    // Resolve binary operators. At this point, we have only expressions interspersed with operators.

    let join_binary_at = |linker: &mut ImperativeLinker, arguments: &mut Vec<ExpressionID>, alias: &String, i: usize| -> Result<(), LinkError> {
        let lhs = arguments.remove(i);
        let rhs = arguments.remove(i);
        let operator = scope.resolve(Environment::Global, &alias)?;
        let overload = scope.resolve(Environment::Global, alias)?.as_function_overload()?;

        Ok(arguments.insert(
            i,
            linker.link_function_call(&overload.functions(), &overload.name, vec![ParameterKey::Positional, ParameterKey::Positional], vec![lhs, rhs], scope)?
        ))
    };

    for (group, group_operators) in &scope.precedence_groups {
        match group.associativity {
            OperatorAssociativity::Left => {
                // Iterate left to right
                let mut i = 0;
                while i < keywords.len() {
                    if let Some(alias) = group_operators.get(&keywords[i]) {
                        keywords.remove(i);
                        join_binary_at(linker, &mut arguments, alias, i)?;
                    }
                    else {
                        i += 1;  // Skip
                    }
                }
            }
            OperatorAssociativity::Right => {
                // Iterate right to left
                let mut i = keywords.len();
                while i > 0 {
                    i -= 1;
                    if let Some(alias) = group_operators.get(&keywords[i]) {
                        keywords.remove(i);
                        join_binary_at(linker, &mut arguments, alias, i)?;
                    }
                }
            }
            OperatorAssociativity::None => {
                // Iteration direction doesn't matter here.
                let mut i = 0;
                while i < keywords.len() {
                    if let Some(alias) = group_operators.get(&keywords[i]) {
                        if i + 1 < group_operators.len() && group_operators.contains_key(&keywords[i + 1]) {
                            panic!("Cannot parse two neighboring {} operators because no associativity is defined.", &keywords[i]);
                        }

                        keywords.remove(i);
                        join_binary_at(linker, &mut arguments, alias, i)?;
                    }

                    i += 1;
                }
            }
            OperatorAssociativity::ConjunctivePairs => {
                // Iteration direction doesn't matter here.
                let mut i = 0;
                while i < keywords.len() {
                    if !group_operators.contains_key(&keywords[i]) {
                        // Skip
                        i += 1;
                        continue;
                    }

                    if i + 1 >= keywords.len() || !group_operators.contains_key(&keywords[i + 1]) {
                        // Just one operation; let's use a binary operator.
                        join_binary_at(linker, &mut arguments, &group_operators[&keywords.remove(i)], i)?;
                        continue;
                    }

                    // More than one operation; Let's build a pairwise operation!
                    // We can start with the first two operators and 3 arguments of which we
                    // know they belong to the operation.
                    let mut group_arguments = vec![
                        arguments.remove(i), arguments.remove(i), arguments.remove(i)
                    ];
                    let mut group_operators = vec![
                        keywords.remove(i), keywords.remove(i)
                    ];

                    while i < keywords.len() && group_operators.contains(&keywords[i]) {
                        // Found one more! Yay!
                        group_arguments.push(arguments.remove(i));
                        group_operators.push(keywords.remove(i));
                    }

                    // Let's wrap this up.
                    arguments.insert(i, linker.link_conjunctive_pairs(
                        group_arguments,
                        todo!("Resolve group_operators to overloads")
                    )?);
                }
            }
            // Unary operators are already resolved at this stage.
            OperatorAssociativity::LeftUnary => {}
        }

        if keywords.len() == 0 {
            // We can return early
            return Ok(arguments.into_iter().next().unwrap())
        }
    }

    if keywords.len() > 0 {
        panic!("Unrecognized binary operator pattern(s); did you forget an import? Offending Operators: {:?}", &keywords);
    }

    Ok(arguments.into_iter().next().unwrap())
}
