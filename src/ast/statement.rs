use std::fmt::{Display, Error, Formatter};

use crate::ast::conformance::TraitConformanceDeclaration;
use crate::ast::expression::Expression;
use crate::ast::function::Function;
use crate::ast::trait_::TraitDefinition;
use crate::program::allocation::Mutability;

#[derive(Eq, PartialEq, Clone)]
pub enum Statement {
    VariableDeclaration {
        mutability: Mutability,
        identifier: String,
        type_declaration: Option<Box<Expression>>,
        assignment: Option<Box<Expression>>
    },
    VariableUpdate { target: Box<Expression>, new_value: Box<Expression> },
    Expression(Box<Expression>),
    Return(Option<Box<Expression>>),
    FunctionDeclaration(Box<Function>),
    Trait(Box<TraitDefinition>),
    Conformance(Box<TraitConformanceDeclaration>),
}

impl Display for Statement {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Statement::VariableDeclaration { mutability, identifier, type_declaration, assignment} => {
                let mutability_string = mutability.variable_declaration_keyword();
                write!(fmt, "{} {}", mutability_string, identifier)?;
                if let Some(type_declaration) = type_declaration {
                    write!(fmt, " '{}", type_declaration)?;
                }
                if let Some(assignment) = assignment {
                    write!(fmt, " = {}", assignment)?;
                }
                Ok(())
            },
            Statement::VariableUpdate { target, new_value } => {
                write!(fmt, "upd {} = {}", target, new_value)
            },
            Statement::Return(Some(expression)) => write!(fmt, "return {}", expression),
            Statement::Return(None) => write!(fmt, "return"),
            Statement::Expression(ref expression) => write!(fmt, "{}", expression),
            Statement::FunctionDeclaration(function) => write!(fmt, "{}", function),
            Statement::Trait(trait_) => write!(fmt, "{}", trait_),
            Statement::Conformance(conformance) => write!(fmt, "{}", conformance),
        }
    }
}
