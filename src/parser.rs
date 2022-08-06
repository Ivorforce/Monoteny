use crate::tenlang_grammar;

pub mod associativity;
pub mod abstract_syntax;

use abstract_syntax::*;
use crate::program::builtins::TenLangBuiltins;
use crate::program::scopes;

pub fn parse_program(content: &String, scope: &scopes::Hierarchy, builtins: &TenLangBuiltins) -> Program {
    let mut program: Program = tenlang_grammar::ProgramParser::new()
        .parse(content.as_str())
        .unwrap();

    // Now comes the evil contextual parsing.

    for statement in program.global_statements.iter_mut() {
        match statement.as_mut() {
            GlobalStatement::FunctionDeclaration(function) => {
                post_parse_function(function);
            }
        }
    }

    program
}

pub fn post_parse_function(function: &mut Function) {
    function.body = function.body.iter()
        .map(|x| post_parse_statement(x.as_ref()))
        .collect()
}

pub fn post_parse_statement(statement: &Statement) -> Box<Statement> {
    Box::new(match statement {
        Statement::VariableDeclaration { mutability, identifier, type_declaration, expression } => {
            Statement::VariableDeclaration {
                mutability: mutability.clone(),
                identifier: identifier.clone(),
                type_declaration: type_declaration.as_ref().map(|x| post_parse_type_declaration(x)),
                expression: post_parse_expression(expression.as_ref())
            }
        },
        Statement::VariableAssignment { variable_name, new_value } => {
            Statement::VariableAssignment { variable_name: variable_name.clone(), new_value: post_parse_expression(new_value.as_ref()) }
        }
        Statement::Expression(expression) => {
            Statement::Expression(post_parse_expression(expression.as_ref()))
        }
        Statement::Return(expression) => {
            Statement::Return(expression.as_ref().map(|x| post_parse_expression(x)))
        }
    })
}

pub fn post_parse_type_declaration(declaration: &TypeDeclaration) -> Box<TypeDeclaration> {
    Box::new(match declaration {
        TypeDeclaration::Identifier(i) => TypeDeclaration::Identifier(i.clone()),
        TypeDeclaration::NDArray(atom, shape) => {
            TypeDeclaration::NDArray(
                post_parse_type_declaration(
                    atom.as_ref()),
                shape.iter().map(|x| post_parse_expression(x)).collect()
            )
        }
    })
}

pub fn post_parse_expression(expression: &Expression) -> Box<Expression> {
    Box::new(match expression {
        Expression::Number(v) => Expression::Number(*v),
        Expression::Bool(v) => Expression::Bool(*v),
        Expression::BinaryOperator { lhs, operator, rhs } => {
            Expression::BinaryOperator {
                lhs: post_parse_expression(lhs),
                operator: operator.clone(),
                rhs: post_parse_expression(rhs)
            }
        }
        Expression::UnaryOperator { operator, argument } => {
            Expression::UnaryOperator {
                operator: operator.clone(),
                argument: post_parse_expression(argument)
            }
        }
        Expression::PairAssociativeBinaryOperators { arguments, operators } => {
            Expression::PairAssociativeBinaryOperators {
                arguments: arguments.iter().map(|x| post_parse_expression(x)).collect(),
                operators: operators.clone()
            }
        }
        Expression::UnsortedBinaryOperators { arguments, operators } => {
            let arguments = arguments.iter().map(|x| post_parse_expression(x)).collect();
            let operators = operators.clone();

            // Ok, 100 lines of code just for this call. I've seen worse but this is pretty bad lol.
            return associativity::sort_binary_expressions(arguments, operators)
        }
        Expression::FunctionCall { call_type, callee, arguments } => {
            Expression::FunctionCall {
                call_type: call_type.clone(),
                callee: post_parse_expression(callee),
                arguments: arguments.iter().map(|x| post_parse_passed_argument(x.as_ref())).collect()
            }
        }
        Expression::MemberLookup { target, member_name } => {
            Expression::MemberLookup {
                target: post_parse_expression(target),
                member_name: member_name.clone()
            }
        }
        Expression::VariableLookup(n) => {
            Expression::VariableLookup(n.clone())
        }
        Expression::ArrayLiteral(arguments) => {
            Expression::ArrayLiteral(arguments.iter().map(|x| post_parse_expression(x)).collect())
        }
        Expression::StringLiteral(s) => {
            Expression::StringLiteral(s.clone())
        }
    })
}

pub fn post_parse_passed_argument(argument: &PassedArgument) -> Box<PassedArgument> {
    Box::new(PassedArgument {
        key: argument.key.clone(),
        value: post_parse_expression(&argument.value)
    })
}
