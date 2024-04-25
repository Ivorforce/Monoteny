use std::fmt::{Display, Error, Formatter};
use std::ops::{Deref, DerefMut};

use itertools::Itertools;

use crate::error::{RResult, RuntimeError};
use crate::program::allocation::Mutability;
use crate::program::functions::ParameterKey;
use crate::util::fmt::{write_comma_separated_list, write_space_separated_list};
use crate::util::position::Positioned;

// =============================== Global =====================================

#[derive(Eq, PartialEq, Clone)]
pub struct Block {
    pub statements: Vec<Box<Decorated<Positioned<Statement>>>>
}

#[derive(Eq, PartialEq, Clone)]
pub struct Function {
    pub interface: FunctionInterface,
    pub body: Option<Expression>,
}

#[derive(Eq, PartialEq, Clone)]
pub struct FunctionInterface {
    pub expression: Expression,
    pub return_type: Option<Expression>,
}

#[derive(Eq, PartialEq, Clone)]
pub struct KeyedParameter {
    pub key: ParameterKey,
    pub internal_name: String,
    pub param_type: Expression,
}

#[derive(Eq, PartialEq, Clone)]
pub struct TraitDefinition {
    pub name: String,
    pub statements: Vec<Box<Positioned<Statement>>>,
}

#[derive(Eq, PartialEq, Clone)]
pub struct TraitConformanceDeclaration {
    pub declared_for: Expression,
    pub declared: String,
    pub statements: Vec<Box<Positioned<Statement>>>,
}

#[derive(Eq, PartialEq, Clone)]
pub struct IfThenElse {
    pub condition: Expression,
    pub consequent: Expression,
    pub alternative: Option<Expression>,
}


// =============================== Code =====================================

#[derive(Eq, PartialEq, Clone)]
pub enum Statement {
    VariableDeclaration {
        mutability: Mutability,
        identifier: String,
        type_declaration: Option<Expression>,
        assignment: Option<Expression>
    },
    VariableUpdate { target: Expression, new_value: Expression },
    Expression(Expression),
    Return(Option<Expression>),
    FunctionDeclaration(Box<Function>),
    Trait(Box<TraitDefinition>),
    Conformance(Box<TraitConformanceDeclaration>),
    IfThenElse(Box<IfThenElse>),
}

#[derive(Eq, PartialEq, Clone)]
pub struct Expression(Vec<Box<Positioned<Term>>>);

impl Expression {
    pub fn no_errors(&self) -> RResult<()> {
        let errors = self.iter().filter_map(|t| {
            match &t.value {
                Term::Error(e) => Some(e.clone()),
                _ => None
            }
        }).collect_vec();

        if errors.is_empty() {
            return Ok(())
        }
        else {
            return Err(errors)
        }
    }
}

impl Deref for Expression {
    type Target = Vec<Box<Positioned<Term>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Expression {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<Box<Positioned<Term>>>> for Expression {
    fn from(value: Vec<Box<Positioned<Term>>>) -> Self {
        Expression(value)
    }
}

#[derive(Eq, PartialEq, Clone)]
pub enum Term {
    Error(RuntimeError),
    Identifier(String),
    MacroIdentifier(String),
    Dot,
    IntLiteral(String),
    RealLiteral(String),
    Struct(Vec<StructArgument>),
    Array(Vec<ArrayArgument>),
    StringLiteral(Vec<Box<Positioned<StringPart>>>),
    Block(Block),
}

#[derive(Eq, PartialEq, Clone)]
pub struct StructArgument {
    pub key: ParameterKey,
    pub value: Expression,
    pub type_declaration: Option<Expression>,
}

#[derive(Eq, PartialEq, Clone)]
pub struct ArrayArgument {
    pub key: Option<Expression>,
    pub value: Expression,
    pub type_declaration: Option<Expression>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FunctionCallType {
    Call,
    Subscript,
}

#[derive(PartialEq, Eq, Clone)]
pub enum StringPart {
    Literal(String),
    Object(Vec<StructArgument>),
}

#[derive(PartialEq, Eq, Clone)]
pub struct Decorated<T> {
    pub decorations: Vec<ArrayArgument>,
    pub value: T,
}

// =============================== String =====================================

impl Mutability {
    fn variable_declaration_keyword(&self) -> &str {
        match *self {
            Mutability::Mutable => "var",
            Mutability::Immutable => "let",
        }
    }
}

impl Display for Block {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        for item in self.statements.iter() {
            write!(fmt, "{}\n\n", item)?
        };
        return Ok(())
    }
}

impl Display for Function {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "def {}", self.interface)?;

        if let Some(body) = &self.body {
            write!(fmt, " :: {}", body)?;
        }
        return Ok(())
    }
}

impl Display for FunctionInterface {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", &self.expression)?;

        if let Some(return_type) = &self.return_type {
            write!(fmt, " -> {}", return_type)?;
        }

        Ok(())
    }
}

impl Display for TraitDefinition {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "trait {} {{", self.name)?;

        Ok(())
    }
}

impl Display for TraitConformanceDeclaration {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "declare {} is {} {{}} :: ", self.declared_for, self.declared)?;
        Ok(())
    }
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
            Statement::IfThenElse(if_then_else) => {
                write!(fmt, "if {} :: {}", if_then_else.condition, if_then_else.consequent)?;
                if let Some(alternative) = &if_then_else.alternative {
                    write!(fmt, "else :: {}", alternative)?;
                }
                Ok(())
            },
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_space_separated_list(f, &self.iter().map(|b| &b.value).collect_vec())
    }
}

impl Display for Term {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Term::Error(_) => write!(fmt, "ERR"),
            Term::Identifier(s) => write!(fmt, "{}", s),
            Term::MacroIdentifier(s) => write!(fmt, "{}!", s),
            Term::IntLiteral(s) => write!(fmt, "{}", s),
            Term::RealLiteral(s) => write!(fmt, "{}", s),
            Term::StringLiteral(parts) => {
                write!(fmt, "\"")?;
                for part in parts {
                    write!(fmt, "{}", part)?;
                }
                write!(fmt, "\"")
            },
            Term::Struct(arguments) => {
                write!(fmt, "(")?;
                write_comma_separated_list(fmt, arguments)?;
                write!(fmt, ")")
            },
            Term::Array(arguments) => {
                write!(fmt, "[")?;
                write_comma_separated_list(fmt, arguments)?;
                write!(fmt, "]")
            },
            Term::Block(block) => {
                write!(fmt, "{{\n{}}}", block)
            }
            Term::Dot => write!(fmt, "."),
        }
    }
}

impl Display for StructArgument {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{}{}", self.key, self.value)
    }
}

impl Display for ArrayArgument {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        if let Some(key) = &self.key {
            write!(fmt, "{}: ", key)?;
        }
        write!(fmt, "{}", self.value)
    }
}

impl Display for KeyedParameter {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{}{} '{}", self.key, self.internal_name, self.param_type)
    }
}

impl FunctionCallType {
    fn bracket_str(&self) -> &str {
        return match *self {
            FunctionCallType::Call => "()",
            FunctionCallType::Subscript => "[]",
        };
    }
}

impl Display for StringPart {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StringPart::Literal(s) => write!(f, "{}", s),
            StringPart::Object(arguments) => {
                write!(f, "(")?;
                write_comma_separated_list(f, arguments)?;
                write!(f, ")")
            },
        }
    }
}

impl<V: Display> Display for Decorated<V> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "![")?;
        write_comma_separated_list(fmt, &self.decorations)?;
        write!(fmt, "]\n{}", self.value)
    }
}

impl<V> Decorated<V> {
    pub fn decorations_as_vec(&self) -> RResult<Vec<&Expression>> {
        return self.decorations.iter().map(|d| {
            if d.key.is_some() {
                return Err(RuntimeError::new("Decorations cannot have keys.".to_string()))
            }
            if d.type_declaration.is_some() {
                return Err(RuntimeError::new("Decorations cannot have type declarations.".to_string()))
            }

            Ok(&d.value)
        }).try_collect()
    }

    pub fn no_decorations(&self) -> RResult<()> {
        if !self.decorations.is_empty() {
            return Err(RuntimeError::new("Decorations are not supported in this context.".to_string()))
        }

        return Ok(())
    }
}
