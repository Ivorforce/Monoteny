use std::rc::Rc;

use itertools::Itertools;

use crate::error::{ErrInRange, RResult, RuntimeError};
use crate::linker::grammar::{OperatorAssociativity, Token};
use crate::linker::imperative::ImperativeLinker;
use crate::linker::scopes;
use crate::parser::ast;
use crate::program::allocation::ObjectReference;
use crate::program::expression_tree::{ExpressionID, ExpressionOperation};
use crate::program::function_object::{FunctionForm, FunctionRepresentation};
use crate::program::functions::{FunctionHead, ParameterKey};
use crate::util::position::Positioned;

pub fn link_expression_to_tokens(linker: &mut ImperativeLinker, syntax: &[Box<Positioned<ast::Term>>], scope: &scopes::Scope) -> RResult<Vec<Positioned<Token>>> {
    // Here's what this function does:
    // We go left to right through all the terms.
    // Many terms can just be evaluated to a token, like int literals or local references.
    // Some terms do something to the previous term. For example, () will either create an empty
    //  struct, or it will call the previous object as function (if any).
    // Some terms also do something to the next term. Mostly, a . will interpret the next term as a
    //  member reference rather than a global.
    let mut tokens = vec![];

    let mut i = 0;
    while i < syntax.len() {
        let ast_token = &syntax[i];
        i += 1;

        match &ast_token.value {
            ast::Term::Error(err) => return Err(vec![err.clone()]),
            ast::Term::Identifier(identifier) => {
                match scope.resolve(scopes::Environment::Global, identifier)? {
                    scopes::Reference::Keyword(keyword) => {
                        tokens.push(ast_token.with_value(
                            Token::Keyword(keyword.clone())
                        ));
                    }
                    scopes::Reference::Local(local) => {
                        let ObjectReference { id, type_, mutability } = local.as_ref();

                        tokens.push(ast_token.with_value(
                            Token::Value(linker.link_unambiguous_expression(
                                vec![],
                                type_,
                                ExpressionOperation::GetLocal(local.clone())
                            )?)
                        ));
                    }
                    scopes::Reference::FunctionOverload(overload) => {
                        tokens.push(ast_token.with_value(
                            match overload.representation.form {
                                FunctionForm::GlobalFunction => {
                                    let next_token = syntax.get(i);

                                    match next_token.map(|t| &t.value) {
                                        Some(ast::Term::Struct(s)) => {
                                            i += 1;

                                            // The next token is a struct; we can immediately call it!
                                            let struct_ = linker.link_struct(scope, s)?;

                                            let expression_id = linker.link_function_call(
                                                overload.functions.iter(),
                                                overload.representation.clone(),
                                                struct_.keys.clone(),
                                                struct_.values.clone(),
                                                scope,
                                                ast_token.position.clone(),
                                            )?;

                                            Token::Value(expression_id)
                                        },
                                        _ => Token::Value(linker.link_function_reference(overload)?),
                                    }
                                }
                                FunctionForm::GlobalImplicit => {
                                    Token::Value(
                                        linker.link_function_call(
                                            overload.functions.iter(),
                                            overload.representation.clone(),
                                            vec![],
                                            vec![],
                                            scope,
                                            ast_token.position.clone()
                                        )?
                                    )
                                }
                                _ => unreachable!(),
                            }
                        ));
                    }
                }
            }
            ast::Term::MacroIdentifier(_) => {
                return Err(RuntimeError::new(format!("Macro not supported here.")))
            }
            ast::Term::Dot => {
                let previous = tokens.last();
                let Some(Token::Value(target)) = previous.map(|v| &v.value) else {
                    return Err(RuntimeError::new(format!("Dot notation requires a preceding object.")))
                };
                let next = syntax.get(i);
                i += 1;
                let Some(ast::Term::Identifier(member)) = next.map(|v| &v.value) else {
                    return Err(RuntimeError::new(format!("Dot notation requires a following identifier.")))
                };

                let overload = scope.resolve(scopes::Environment::Member, member)?
                    .as_function_overload()?;

                // TODO This is almost duplicated code from normal function calls above.
                *tokens.last_mut().unwrap() = ast_token.with_value(match overload.representation.form {
                    FunctionForm::MemberFunction => {
                        let next_token = syntax.get(i);
                        i += 1;

                        match next_token.map(|t| &t.value) {
                            Some(ast::Term::Struct(s)) => {
                                // The next token is a struct; we can immediately call it!
                                let struct_ = linker.link_struct(scope, s)?;

                                let expression_id = linker.link_function_call(
                                    overload.functions.iter(),
                                    overload.representation.clone(),
                                    [ParameterKey::Positional].into_iter().chain(struct_.keys.clone()).collect_vec(),
                                    [target.clone()].into_iter().chain(struct_.values.clone()).collect_vec(),
                                    scope,
                                    ast_token.position.clone(),
                                )?;

                                Token::Value(expression_id)
                            },
                            _ => return Err(RuntimeError::new(format!("Member function references are not yet supported."))),
                        }
                    }
                    FunctionForm::MemberImplicit => {
                        Token::Value(
                            linker.link_function_call(
                                overload.functions.iter(),
                                overload.representation.clone(),
                                vec![ParameterKey::Positional],
                                vec![target.clone()],
                                scope,
                                ast_token.position.clone()
                            )?
                        )
                    }
                    _ => unreachable!(),
                });
            }
            ast::Term::IntLiteral(string) => {
                let string_expression_id = linker.link_string_primitive(string)?;

                tokens.push(ast_token.with_value(
                    Token::Value(linker.link_abstract_function_call(
                        vec![string_expression_id],
                        Rc::clone(&linker.runtime.traits.as_ref().unwrap().ConstructableByIntLiteral),
                        Rc::clone(&linker.runtime.traits.as_ref().unwrap().parse_int_literal_function.target),
                        scope.trait_conformance.clone(),
                        ast_token.position.clone(),
                    )?)
                ));
            }
            ast::Term::RealLiteral(string) => {
                let string_expression_id = linker.link_string_primitive(string)?;

                tokens.push(ast_token.with_value(
                    Token::Value(linker.link_abstract_function_call(
                        vec![string_expression_id],
                        Rc::clone(&linker.runtime.traits.as_ref().unwrap().ConstructableByRealLiteral),
                        Rc::clone(&linker.runtime.traits.as_ref().unwrap().parse_real_literal_function.target),
                        scope.trait_conformance.clone(),
                        ast_token.position.clone(),
                    )?)
                ));
            }
            ast::Term::StringLiteral(parts) => {
                tokens.push(ast_token.with_value(
                    Token::Value(linker.link_string_literal(scope, ast_token, parts)?)
                ))
            }
            ast::Term::Struct(s) => {
                let struct_ = linker.link_struct(scope, s)?;

                let previous = tokens.last();
                match previous.map(|v| &v.value) {
                    Some(Token::Value(expression)) => {
                        // Call previous token
                        let overload = scope
                            .resolve(scopes::Environment::Member, "call_as_function").err_in_range(&ast_token.position)?
                            .as_function_overload().err_in_range(&ast_token.position)?;

                        let expression_id = linker.link_function_call(
                            overload.functions.iter(),
                            overload.representation.clone(),
                            [&ParameterKey::Positional].into_iter().chain(&struct_.keys).cloned().collect(),
                            [expression].into_iter().chain(&struct_.values).cloned().collect(),
                            scope,
                            ast_token.position.clone(),
                        )?;

                        *tokens.last_mut().unwrap() = ast_token.with_value(Token::Value(expression_id));
                    }
                    _ => {
                        // No previous token; Use struct as value.
                        if &struct_.keys[..] == &[ParameterKey::Positional] {
                            let expression_id = *struct_.values.iter().exactly_one().unwrap();
                            tokens.push(ast_token.with_value(
                                Token::Value(expression_id)
                            ));
                        } else {
                            return Err(RuntimeError::new(String::from("Anonymous struct literals are not yet supported.")))
                        }
                    }
                }
            }
            ast::Term::Array(a) => {
                let values = a.iter().map(|x| {
                    linker.link_expression_with_type(&x.value, &x.type_declaration, scope)
                }).try_collect()?;

                let previous = tokens.last();
                match previous.map(|v| &v.value) {
                    Some(Token::Value(expression)) => {
                        return Err(RuntimeError::new(String::from("Object subscript is not yet supported.")))
                    }
                    _ => {
                        let supertype = linker.types.merge_all(&values)?.clone();
                        return Err(RuntimeError::new(String::from("Array literals are not yet supported.")))

                        // tokens.push(ast_token.with_value(
                        //     Token::Expression(linker.link_unambiguous_expression(
                        //         vec![],
                        //         &TypeProto::monad(TypeProto::unit(TypeUnit::Generic(supertype))),
                        //         ExpressionOperation::ArrayLiteral
                        //     )?)
                        // ));
                    }
                }
            }
            ast::Term::Block(statements) => {
                tokens.push(ast_token.with_value(
                    Token::Value(linker.link_block(statements, &scope)?)
                ))
            }
        }
    }

    Ok(tokens)
}

pub fn link_patterns(mut tokens: Vec<Positioned<Token>>, scope: &scopes::Scope, linker: &mut ImperativeLinker) -> RResult<ExpressionID> {
    let mut values: Vec<Positioned<ExpressionID>> = vec![];
    let mut keywords: Vec<String> = vec![];

    let final_ptoken = tokens.remove(tokens.len() - 1);
    if let Token::Value(expression) = &final_ptoken.value {
        values.push(final_ptoken.with_value(*expression));
    }
    else {
        return Err(RuntimeError::new_in_range(String::from("Expected expression."), final_ptoken.position))
    }

    // Reduce all unary operators, and build interspersed arguments / operators list.
    if let Some((group, left_unary_operators)) = &scope.grammar.groups_and_keywords.iter().next() {
        if group.associativity != OperatorAssociativity::LeftUnary {
            todo!("Left Unary operators must be first for now.");
        }

        while let Some(ptoken) = tokens.pop() {
            let Token::Keyword(keyword) = ptoken.value else {
                return Err(RuntimeError::new(String::from("Got two consecutive values; expected an operator in between.")))
            };

            if let Some(ptoken) = tokens.last() {
                if let Token::Value(expression) = &ptoken.value {
                    // Binary Operator, because left of operator is a value
                    values.insert(0, ptoken.with_value(*expression));
                    keywords.insert(0, keyword);
                    tokens.pop();

                    continue
                }
            }

            // Unary operator, because left of operator is an operator

            let function_head = &left_unary_operators[&keyword];
            let argument = values.remove(0);
            let expression_id = linker.link_function_call(
                [function_head].into_iter(),
                FunctionRepresentation { name: "fn".to_string(), form: FunctionForm::GlobalFunction },
                vec![ParameterKey::Positional],
                vec![argument.value],
                scope,
                ptoken.position
            )?;
            values.insert(0, argument.with_value(expression_id));
        }
    }

    if values.len() == 1 {
        // Just one argument, we can shortcut!
        return Ok(values.remove(0).value)
    }

    // Resolve binary operators. At this point, we have only expressions interspersed with operators.

    let join_binary_at = |linker: &mut ImperativeLinker, arguments: &mut Vec<Positioned<ExpressionID>>, function_head: &Rc<FunctionHead>, i: usize| -> RResult<()> {
        let lhs = arguments.remove(i);
        let rhs = arguments.remove(i);

        let range = lhs.position.start..rhs.position.end;

        Ok(arguments.insert(
            i,
            Positioned {
                position: range.clone(),
                value: linker.link_function_call(
                    [function_head].into_iter(),
                    FunctionRepresentation { name: "fn".to_string(), form: FunctionForm::GlobalFunction },
                    vec![ParameterKey::Positional, ParameterKey::Positional],
                    vec![lhs.value, rhs.value],
                    scope,
                    range
                )?,
            }
        ))
    };

    for (group, group_operators) in scope.grammar.groups_and_keywords.iter() {
        match group.associativity {
            OperatorAssociativity::Left => {
                // Iterate left to right
                let mut i = 0;
                while i < keywords.len() {
                    if let Some(function_head) = group_operators.get(&keywords[i]) {
                        keywords.remove(i);
                        join_binary_at(linker, &mut values, function_head, i)?;
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
                        join_binary_at(linker, &mut values, alias, i)?;
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
                        join_binary_at(linker, &mut values, alias, i)?;
                    }

                    i += 1;
                }
            }
            OperatorAssociativity::LeftConjunctivePairs => {
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
                        join_binary_at(linker, &mut values, &group_operators[&keywords.remove(i)], i)?;
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
                    values.insert(i, linker.link_conjunctive_pairs(
                        group_arguments,
                        todo!("Resolve group_operators to overloads")
                    )?);
                }
            }
            // Unary operators are already resolved at this stage.
            OperatorAssociativity::LeftUnary => {},
            OperatorAssociativity::RightUnary => todo!(),
        }

        if keywords.len() == 0 {
            // We can return early
            return Ok(values.iter().exactly_one().unwrap().value)
        }
    }

    if keywords.len() > 0 {
        panic!("Unrecognized binary operator pattern(s); did you forget an import? Offending Operators: {:?}", &keywords);
    }

    Ok(values.into_iter().exactly_one().unwrap().value)
}
