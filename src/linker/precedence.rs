use std::hash::{Hash, Hasher};
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use uuid::Uuid;
use crate::error::{ErrInRange, RResult, RuntimeError};
use crate::linker::imperative::ImperativeLinker;
use crate::linker::scopes;
use crate::linker::scopes::Environment;
use crate::program::computation_tree::{ExpressionID, ExpressionOperation};
use crate::program::functions::{FunctionOverload, ParameterKey};
use crate::program::r#struct::Struct;
use crate::program::types::{TypeProto, TypeUnit};
use crate::util::position::{Positioned, positioned};

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
    AnonymousStruct(Struct),
    AnonymousArray { keys: Vec<Option<ExpressionID>>, values: Vec<ExpressionID> },
    FunctionReference { overload: Rc<FunctionOverload>, target: Option<ExpressionID> },
}

pub fn link_patterns(mut tokens: Vec<Positioned<Token>>, scope: &scopes::Scope, linker: &mut ImperativeLinker) -> RResult<ExpressionID> {
    // Resolve structs and array literals
    let mut i = 0;
    for _ in 0 .. tokens.len() {
        let ptoken = &tokens[i];

        match &ptoken.value {
            Token::AnonymousStruct(struct_) => {
                let previous = tokens.get(i - 1);
                match previous.map(|v| &v.value) {
                    Some(Token::FunctionReference { overload, target }) => {
                        let expression_id = linker.link_function_call(
                            &overload.functions(),
                            &overload.name,
                            target.iter().map(|_| &ParameterKey::Positional).chain(&struct_.keys).cloned().collect(),
                            target.iter().chain(&struct_.values).cloned().collect(),
                            scope,
                            ptoken.position.clone(),
                        )?;

                        tokens[i] = positioned(Token::Expression(expression_id), previous.unwrap().position.start, ptoken.position.end);
                        tokens.remove(i - 1);
                    }
                    Some(Token::Expression(expression)) => {
                        let overload = scope
                            .resolve(Environment::Member, "call_as_function").err_in_range(&ptoken.position)?
                            .as_function_overload().err_in_range(&ptoken.position)?;

                        let expression_id = linker.link_function_call(
                            &overload.functions(),
                            &overload.name,
                            [&ParameterKey::Positional].into_iter().chain(&struct_.keys).cloned().collect(),
                            [expression].into_iter().chain(&struct_.values).cloned().collect(),
                            scope,
                            ptoken.position.clone(),
                        )?;

                        tokens[i] = ptoken.with_value(Token::Expression(expression_id));
                        tokens.remove(i - 1);
                    }
                    _ => {
                        if &struct_.keys[..] == &[ParameterKey::Positional] {
                            let expression_id = *struct_.values.iter().exactly_one().unwrap();
                            tokens[i] = ptoken.with_value(Token::Expression(expression_id));
                        }
                        else {
                            return Err(RuntimeError::new(String ::from("Anonymous struct literals are not yet supported.")))
                        }
                    }
                }
            }
            Token::AnonymousArray { keys, values } => {
                let previous = tokens.get(i - 1);
                match previous.map(|v| &v.value) {
                    Some(Token::FunctionReference { overload, target }) => {
                        return Err(RuntimeError::new(String::from("Functions with subscript form are not yet supported.")))
                    }
                    Some(Token::Expression(expression)) => {
                        return Err(RuntimeError::new(String::from("Object subscript is not yet supported.")))
                    }
                    _ => {
                        return Err(RuntimeError::new(String::from("Array literals are not yet supported.")))

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
        let ptoken = &tokens[i];
        if let Token::FunctionReference { overload, target } = &ptoken.value {
            match overload.pointers.len() {
                1 => {
                    let ref_ = overload.pointers.iter().next().unwrap();
                    let function = ref_.as_function_head()?;

                    let expression_id = linker.link_unambiguous_expression(
                        vec![],
                        &TypeProto::unit(TypeUnit::Function(Rc::clone(function))),
                        ExpressionOperation::VariableLookup(ref_.clone())
                    )?;

                    tokens[i] = ptoken.with_value(Token::Expression(expression_id));
                }
                _ => Err(RuntimeError::new(
                    String::from("References to overloaded functions are not yet supported (need syntax to distinguish which to choose).")
                ))?,
            }
        }
    }

    let mut arguments: Vec<Positioned<ExpressionID>> = vec![];
    let mut keywords: Vec<String> = vec![];

    let final_ptoken = tokens.remove(tokens.len() - 1);
    if let Token::Expression(expression) = &final_ptoken.value {
        arguments.push(final_ptoken.with_value(*expression));
    }
    else {
        return Err(RuntimeError::new(String::from("Expression missing the final argument.")).in_range(final_ptoken.position))
    }

    // Reduce all unary operators, and build interspersed arguments / operators list.
    let left_unary_operators = &scope.precedence_groups[0].1;
    while !tokens.is_empty() {
        let ptoken = tokens.remove(tokens.len() - 1);
        guard!(let Token::Keyword(keyword) = ptoken.value else {
            return Err(RuntimeError::new(String::from("Expecting an operator but got an expression.")))
        });

        if let Some(ptoken) = tokens.get(tokens.len() - 1) {
            if let Token::Expression(expression) = &ptoken.value {
                // Binary Operator, because left of operator is an expression!
                arguments.insert(0, ptoken.with_value(*expression));
                keywords.insert(0, keyword);
                tokens.remove(tokens.len() - 1);

                continue
            }
        }

        let overload = scope.resolve(Environment::Global, &left_unary_operators[&keyword])?.as_function_overload()?;

        // Unary operator, because left of operator is an operator!
        let argument = arguments.remove(0);
        let expression_id = linker.link_function_call(&overload.functions(), &overload.name, vec![ParameterKey::Positional], vec![argument.value], scope, ptoken.position)?;
        arguments.insert(0, argument.with_value(expression_id));
    }

    if arguments.len() == 1 {
        // Just one argument, we can shortcut!
        return Ok(arguments.remove(0).value)
    }

    // Resolve binary operators. At this point, we have only expressions interspersed with operators.

    let join_binary_at = |linker: &mut ImperativeLinker, arguments: &mut Vec<Positioned<ExpressionID>>, alias: &str, i: usize| -> RResult<()> {
        let lhs = arguments.remove(i);
        let rhs = arguments.remove(i);
        let operator = scope.resolve(Environment::Global, &alias)?;
        let overload = scope.resolve(Environment::Global, alias)?.as_function_overload()?;

        let range = lhs.position.start..rhs.position.end;

        Ok(arguments.insert(
            i,
            Positioned {
                position: range.clone(),
                value: linker.link_function_call(&overload.functions(), &overload.name, vec![ParameterKey::Positional, ParameterKey::Positional], vec![lhs.value, rhs.value], scope, range)?,
            }
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
            return Ok(arguments.iter().exactly_one().unwrap().value)
        }
    }

    if keywords.len() > 0 {
        panic!("Unrecognized binary operator pattern(s); did you forget an import? Offending Operators: {:?}", &keywords);
    }

    Ok(arguments.into_iter().exactly_one().unwrap().value)
}
