use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use itertools::Itertools;

use crate::ast;
use crate::error::{RResult, RuntimeError};
use crate::parser::grammar::{Grammar, OperatorAssociativity};
use crate::util::position::Positioned;

pub enum Value<'a, Function> {
    Operation(Function, Vec<Box<Positioned<Self>>>),
    Identifier(&'a String),
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
    Keyword(&'a String),
    Value(Box<Positioned<Value<'a, Function>>>),
}

pub fn parse_to_tokens<'a, Function: Clone + PartialEq + Eq + Hash + Debug>(syntax: &'a[Box<Positioned<ast::Term>>], grammar: &'a Grammar<Function>) -> RResult<Vec<Positioned<Token<'a, Function>>>> {
    let mut tokens = vec![];

    let mut i = 0;
    while i < syntax.len() {
        let ast_token = &syntax[i];
        i += 1;

        match &ast_token.value {
            ast::Term::Error(err) => Err(err.clone().to_array())?,
            ast::Term::Identifier(identifier) => {
                if grammar.keywords.contains(identifier) {
                    tokens.push(ast_token.with_value(Token::Keyword(identifier)));
                }
                else {
                    tokens.push(ast_token.with_value(Token::Value(Box::new(ast_token.with_value(Value::Identifier(identifier))))));
                }
            }
            ast::Term::MacroIdentifier(_) => {
                return Err(RuntimeError::error("Macro not supported here.").to_array())
            }
            ast::Term::Dot => {
                let Some(target_token) = tokens.pop() else {
                    return Err(RuntimeError::error("Dot notation requires a preceding object.").to_array())
                };
                let Token::Value(target) = target_token.value else {
                    return Err(RuntimeError::error("Dot notation requires a preceding object.").to_array())
                };

                let Some(next) = syntax.get(i) else {
                    return Err(RuntimeError::error("Dot notation requires a following identifier.").to_array())
                };
                let ast::Term::Identifier(member) = &next.value else {
                    return Err(RuntimeError::error("Dot notation requires a following identifier.").to_array())
                };

                i += 1;
                tokens.push(Positioned {
                    position: target_token.position,
                    value: Token::Value(Box::new(Positioned {
                        position: next.position.clone(),
                        value: Value::MemberAccess(target, member)
                    }))
                });
            }
            ast::Term::IntLiteral(string) => {
                tokens.push(ast_token.with_value(Token::Value(Box::new(ast_token.with_value(Value::IntLiteral(string))))));
            }
            ast::Term::RealLiteral(string) => {
                tokens.push(ast_token.with_value(Token::Value(Box::new(ast_token.with_value(Value::RealLiteral(string))))));
            }
            ast::Term::StringLiteral(parts) => {
                tokens.push(ast_token.with_value(Token::Value(Box::new(ast_token.with_value(Value::StringLiteral(parts))))));
            }
            ast::Term::Struct(s) => {
                if let Some(Token::Value(_)) = tokens.last().map(|t| &t.value) {
                    // Previous token; we've got a direct call!
                    let Token::Value(previous) = tokens.pop().unwrap().value else { panic!() };
                    let previous_position = previous.position.clone();

                    tokens.push(Positioned {
                        position: previous_position.start..ast_token.position.end,
                        value: Token::Value(Box::new(ast_token.with_value(Value::FunctionCall(previous, s))))
                    });
                    continue;
                }

                tokens.push(ast_token.with_value(Token::Value(Box::new(ast_token.with_value(Value::StructLiteral(s))))));
            }
            ast::Term::Array(array) => {
                if let Some(Token::Value(_)) = tokens.last().map(|t| &t.value) {
                    // Previous token; we've got a direct call!
                    let Token::Value(previous) = tokens.pop().unwrap().value else { panic!() };

                    tokens.push(ast_token.with_value(Token::Value(Box::new(ast_token.with_value(Value::Subscript(previous, array))))));
                    continue;
                }

                tokens.push(ast_token.with_value(Token::Value(Box::new(ast_token.with_value(Value::ArrayLiteral(array))))));
            }
            ast::Term::Block(block) => {
                tokens.push(ast_token.with_value(Token::Value(Box::new(ast_token.with_value(Value::Block(block))))));
            }
            ast::Term::IfThenElse(if_then_else) => {
                tokens.push(ast_token.with_value(Token::Value(Box::new(ast_token.with_value(Value::IfThenElse(if_then_else))))));
            }
        }
    }

    Ok(tokens)
}

pub fn parse_unary<'a, Function: Clone + PartialEq + Eq + Hash + Debug>(mut tokens: Vec<Positioned<Token<'a, Function>>>, functions: Option<&'a HashMap<String, Function>>) -> RResult<(Vec<Box<Positioned<Value<'a, Function>>>>, Vec<&'a str>)> {
    let mut values: Vec<Box<Positioned<Value<Function>>>> = vec![];
    let mut keywords: Vec<&str> = vec![];

    let final_ptoken = tokens.remove(tokens.len() - 1);
    if let Token::Value(value) = final_ptoken.value {
        values.push(value);
    }
    else {
        return Err(RuntimeError::error("Expected expression.").in_range(final_ptoken.position).to_array())
    }

    if let Some(functions) = functions {
        while let Some(ptoken) = tokens.pop() {
            let Token::Keyword(keyword) = ptoken.value else {
                return Err(
                    RuntimeError::error("Found two consecutive values; expected an operator in between.")
                        .in_range(ptoken.position.end..values.last().unwrap().position.start)
                        .to_array()
                )
            };

            if let Some(Token::Value(_)) = tokens.last().map(|t| &t.value) {
                let Token::Value(value) = tokens.pop().unwrap().value else { panic!() };

                // Binary Operator keyword, because left of operator is a value
                values.insert(0, value);
                keywords.insert(0, keyword);

                continue
            }

            // Unary operator, because left of operator is an operator
            let argument = values.remove(0);
            values.insert(0, Box::new(ptoken.with_value(Value::Operation(functions[keyword].clone(), vec![argument]))));
        }
    }

    return Ok((values, keywords))
}

pub fn resolve_expression<'a, Function: Clone + PartialEq + Eq + Hash + Debug>(syntax: &'a[Box<Positioned<ast::Term>>], grammar: &'a Grammar<Function>) -> RResult<Box<Positioned<Value<'a, Function>>>> {
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
    let join_binary_at = |arguments: &mut Vec<Box<Positioned<Value<Function>>>>, function: &Function, i: usize| -> RResult<()> {
        let lhs = arguments.remove(i);
        let rhs = arguments.remove(i);

        let range = lhs.position.start..rhs.position.end;

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
                    if let Some(function_head) = group_operators.get(keywords[i]) {
                        keywords.remove(i);
                        join_binary_at(&mut values, function_head, i)?;
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
                    if let Some(alias) = group_operators.get(keywords[i]) {
                        keywords.remove(i);
                        join_binary_at(&mut values, alias, i)?;
                    }
                }
            }
            OperatorAssociativity::None => {
                // Iteration direction doesn't matter here.
                let mut i = 0;
                while i < keywords.len() {
                    if let Some(alias) = group_operators.get(keywords[i]) {
                        if i + 1 < group_operators.len() && group_operators.contains_key(keywords[i + 1]) {
                            panic!("Cannot parse two neighboring {} operators because no associativity is defined.", keywords[i]);
                        }

                        keywords.remove(i);
                        join_binary_at(&mut values, alias, i)?;
                    }

                    i += 1;
                }
            }
            OperatorAssociativity::LeftConjunctivePairs => {
                // Iteration direction doesn't matter here.
                let mut i = 0;
                while i < keywords.len() {
                    if !group_operators.contains_key(keywords[i]) {
                        // Skip
                        i += 1;
                        continue;
                    }

                    if i + 1 >= keywords.len() || !group_operators.contains_key(keywords[i + 1]) {
                        // Just one operation; let's use a binary operator.
                        join_binary_at(&mut values, &group_operators[keywords.remove(i)], i)?;
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
