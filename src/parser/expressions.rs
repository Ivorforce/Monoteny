use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Range;

use itertools::Itertools;

use crate::ast;
use crate::error::{RResult, RuntimeError};
use crate::parser::grammar::{Grammar, OperatorAssociativity};
use crate::util::position::Positioned;

pub enum Value<'a, Function> {
    Operation(Function, Vec<Box<Positioned<Self>>>),
    Identifier(&'a String),
    MacroIdentifier(&'a String),
    RealLiteral(&'a String),
    IntLiteral(&'a String),
    StringLiteral(&'a Vec<Box<Positioned<ast::StringPart>>>),
    StructLiteral(&'a ast::Struct),
    ArrayLiteral(&'a ast::Array),
    Block(&'a ast::Block),
    MemberAccess(Box<Positioned<Self>>, &'a String),
    FunctionCall(Box<Positioned<Self>>, &'a ast::Struct),
    Subscript(Box<Positioned<Self>>, &'a ast::Array),
    IfThenElse(&'a ast::IfThenElse),
}

pub enum Token<'a, Function> {
    Keyword(Positioned<&'a String>),
    Value(Box<Positioned<Value<'a, Function>>>),
}

pub fn parse_to_tokens<'a, Function: Clone + PartialEq + Eq + Hash + Debug>(syntax: &'a[Box<Positioned<ast::Term>>], grammar: &'a Grammar<Function>) -> RResult<Vec<Token<'a, Function>>> {
    let mut tokens: Vec<Token<'a, Function>> = vec![];

    let mut i = 0;
    while i < syntax.len() {
        let ast_token = &syntax[i];
        i += 1;

        match &ast_token.value {
            ast::Term::Error(err) => Err(err.clone().to_array())?,
            ast::Term::Identifier(identifier) => {
                if grammar.keywords.contains(identifier) {
                    tokens.push(Token::Keyword(ast_token.with_value(identifier)));
                }
                else {
                    tokens.push(Token::Value(Box::new(ast_token.with_value(Value::Identifier(identifier)))));
                }
            }
            ast::Term::MacroIdentifier(identifier) => {
                tokens.push(Token::Value(Box::new(ast_token.with_value(Value::MacroIdentifier(identifier)))));
            }
            ast::Term::Dot => {
                let Some(Token::Value(target)) = tokens.pop() else {
                    return Err(RuntimeError::error("Dot notation requires a preceding object.").in_range(ast_token.position.clone()).to_array())
                };

                let Some(next_token) = syntax.get(i) else {
                    return Err(RuntimeError::error("Dot notation requires a following identifier.").in_range(ast_token.position.clone()).to_array())
                };
                let ast::Term::Identifier(member) = &next_token.value else {
                    return Err(RuntimeError::error("Dot notation requires a following identifier.").in_range(ast_token.position.clone()).to_array())
                };

                i += 1;
                tokens.push(Token::Value(Box::new(next_token.with_value(Value::MemberAccess(target, member)))));
            }
            ast::Term::IntLiteral(string) => {
                tokens.push(Token::Value(Box::new(ast_token.with_value(Value::IntLiteral(string)))));
            }
            ast::Term::RealLiteral(string) => {
                tokens.push(Token::Value(Box::new(ast_token.with_value(Value::RealLiteral(string)))));
            }
            ast::Term::StringLiteral(parts) => {
                tokens.push(Token::Value(Box::new(ast_token.with_value(Value::StringLiteral(parts)))));
            }
            ast::Term::Struct(s) => {
                if let Some(Token::Value(_)) = tokens.last() {
                    // Previous token; we've got a call!
                    let Token::Value(previous) = tokens.pop().unwrap() else { panic!() };

                    let position = previous.position.start..ast_token.position.end;
                    tokens.push(Token::Value(
                        Box::new(Positioned {
                            position,
                            value: Value::FunctionCall(previous, s),
                        })
                    ));
                    continue;
                }

                // No call, just a struct literal.
                tokens.push(Token::Value(Box::new(ast_token.with_value(Value::StructLiteral(s)))));
            }
            ast::Term::Array(array) => {
                if let Some(Token::Value(_)) = tokens.last() {
                    // Previous token; we've got a direct call!
                    let Token::Value(previous) = tokens.pop().unwrap() else { panic!() };

                    tokens.push(Token::Value(Box::new(ast_token.with_value(Value::Subscript(previous, array)))));
                    continue;
                }

                tokens.push(Token::Value(Box::new(ast_token.with_value(Value::ArrayLiteral(array)))));
            }
            ast::Term::Block(block) => {
                tokens.push(Token::Value(Box::new(ast_token.with_value(Value::Block(block)))));
            }
            ast::Term::IfThenElse(if_then_else) => {
                tokens.push(Token::Value(Box::new(ast_token.with_value(Value::IfThenElse(if_then_else)))));
            }
        }
    }

    Ok(tokens)
}

pub fn parse_unary<'a, Function: Clone + PartialEq + Eq + Hash + Debug>(mut tokens: Vec<Token<'a, Function>>, functions: Option<&'a HashMap<String, Function>>) -> RResult<(Vec<Box<Positioned<Value<'a, Function>>>>, Vec<Positioned<&'a str>>)> {
    let mut values: Vec<Box<Positioned<Value<Function>>>> = vec![];
    let mut keywords: Vec<Positioned<&'a str>> = vec![];

    match tokens.pop() {
        Some(Token::Value(value)) => values.push(value),
        Some(Token::Keyword(keyword)) => {
            return Err(RuntimeError::error("Expected value.").in_range(keyword.position).to_array())
        }
        None => {
            return Err(RuntimeError::error("Expected expression.").to_array())
        }
    }

    if let Some(functions) = functions {
        while let Some(token) = tokens.pop() {
            let Token::Keyword(keyword) = &token else {
                let Token::Value(value) = &token else { panic!() };
                return Err(
                    RuntimeError::error("Found two consecutive values; expected an operator in between.")
                        .in_range(value.position.end..values.last().unwrap().position.start)
                        .to_array()
                )
            };

            if let Some(Token::Value(_)) = tokens.last() {
                let Token::Value(value) = tokens.pop().unwrap() else { panic!() };

                // Binary Operator keyword, because left of operator is a value
                values.insert(0, value);
                keywords.insert(0, keyword.with_value(keyword.value.as_str()));

                continue
            }

            // Unary operator, because left of operator is an operator
            let argument = values.remove(0);
            values.insert(0, Box::new(keyword.with_value(Value::Operation(functions[keyword.value.as_str()].clone(), vec![argument]))));
        }
    }

    return Ok((values, keywords))
}

pub fn parse<'a, Function: Clone + PartialEq + Eq + Hash + Debug>(syntax: &'a[Box<Positioned<ast::Term>>], grammar: &'a Grammar<Function>) -> RResult<Box<Positioned<Value<'a, Function>>>> {
    // Here's what this function does:
    // We go left to right through all the terms.
    // Many terms can just be evaluated to a token, like int literals or local references.
    // Some terms do something to the previous term. For example, () will either create an empty
    //  struct, or it will call the previous object as function (if any).
    // Some terms also do something to the next term. Mostly, a . will interpret the next term as a
    //  member reference rather than a global.
    let mut tokens = parse_to_tokens(syntax, grammar)?;

    let left_unary_operators = grammar.groups_and_keywords.iter().next().map(|(group, ops)| {
        if let Some((group, left_unary_operators)) = &grammar.groups_and_keywords.iter().next() {
            if group.associativity != OperatorAssociativity::LeftUnary {
                todo!("Left Unary operators must be first for now.");
            }
        }

        ops
    });

    let (mut values, mut keywords) = parse_unary(tokens, left_unary_operators)?;

    if values.len() == 1 {
        // Just one argument, we can shortcut!
        return Ok(values.remove(0))
    }

    // Resolve binary operators. At this point, we have only expressions interspersed with operators.
    let join_binary_at = |arguments: &mut Vec<Box<Positioned<Value<Function>>>>, function: &Function, range: &Range<usize>, i: usize| -> RResult<()> {
        let lhs = arguments.remove(i);
        let rhs = arguments.remove(i);

        Ok(arguments.insert(
            i,
            Box::new(Positioned {
                position: range.clone(),
                value: Value::Operation(function.clone(), vec![lhs, rhs]),
            })
        ))
    };

    for (group, group_operators) in grammar.groups_and_keywords.iter() {
        match group.associativity {
            OperatorAssociativity::Left => {
                // Iterate left to right
                let mut i = 0;
                while i < keywords.len() {
                    if let Some(function_head) = group_operators.get(keywords[i].value) {
                        let keyword = keywords.remove(i);
                        join_binary_at(&mut values, function_head, &keyword.position, i)?;
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
                    if let Some(alias) = group_operators.get(keywords[i].value) {
                        keywords.remove(i);
                        join_binary_at(&mut values, alias, &keywords[i].position, i)?;
                    }
                }
            }
            OperatorAssociativity::None => {
                // Iteration direction doesn't matter here.
                let mut i = 0;
                while i < keywords.len() {
                    if let Some(alias) = group_operators.get(keywords[i].value) {
                        if i + 1 < group_operators.len() && group_operators.contains_key(keywords[i + 1].value) {
                            panic!("Cannot parse two neighboring {} operators because no associativity is defined.", keywords[i]);
                        }

                        keywords.remove(i);
                        join_binary_at(&mut values, alias, &keywords[i].position, i)?;
                    }

                    i += 1;
                }
            }
            OperatorAssociativity::LeftConjunctivePairs => {
                // Iteration direction doesn't matter here.
                let mut i = 0;
                while i < keywords.len() {
                    if !group_operators.contains_key(keywords[i].value) {
                        // Skip
                        i += 1;
                        continue;
                    }

                    if i + 1 >= keywords.len() || !group_operators.contains_key(keywords[i + 1].value) {
                        // Just one operation; let's use a binary operator.
                        let keyword = keywords.remove(i);
                        join_binary_at(&mut values, &group_operators[keyword.value], &keyword.position, i)?;
                        continue;
                    }

                    // More than one operation; Let's build a pairwise operation!
                    // We can start with the first two operators and 3 arguments of which we
                    // know they belong to the operation.
                    let mut group_arguments = vec![
                        values.remove(i), values.remove(i), values.remove(i)
                    ];
                    let mut group_operators = vec![
                        keywords.remove(i), keywords.remove(i)
                    ];

                    while i < keywords.len() && group_operators.contains(&keywords[i]) {
                        // Found one more! Yay!
                        group_arguments.push(values.remove(i));
                        group_operators.push(keywords.remove(i));
                    }

                    // Let's wrap this up.
                    values.insert(i, todo!("Resolve group_operators to functions"));
                }
            }
            // Unary operators are already resolved at this stage.
            OperatorAssociativity::LeftUnary => {},
            OperatorAssociativity::RightUnary => todo!(),
        }

        if keywords.len() == 0 {
            // We can return early
            assert_eq!(values.len(), 1);
            return Ok(values.pop().unwrap())
        }
    }

    if keywords.len() > 0 {
        panic!("Unrecognized binary operator pattern(s); did you forget an import? Offending Operators: {:?}", &keywords);
    }

    assert_eq!(values.len(), 1);
    return Ok(values.pop().unwrap())
}
