use std::fmt::{Display, Error, Formatter};
use std::ops::{Deref, DerefMut};
use itertools::Itertools;
use crate::error::{RResult, RuntimeError};
use crate::program::functions::ParameterKey;
use crate::program::allocation::Mutability;
use crate::util::fmt::{write_comma_separated_list, write_space_separated_list};
use crate::util::position::Positioned;

// =============================== Global =====================================

#[derive(Eq, PartialEq)]
pub struct Module {
    pub global_statements: Vec<Box<Positioned<Statement>>>
}

#[derive(Eq, PartialEq)]
pub struct Function {
    pub interface: FunctionInterface,
    pub body: Option<Expression>,
}

#[derive(Eq, PartialEq)]
pub enum FunctionInterface {
    Macro(String),
    Explicit {
        identifier: String,
        target_type: Option<Box<Expression>>,
        parameters: Vec<Box<KeyedParameter>>,
        return_type: Option<Expression>,
    }
}

#[derive(Eq, PartialEq)]
pub struct KeyedParameter {
    pub key: ParameterKey,
    pub internal_name: String,
    pub param_type: Expression,
}

#[derive(Eq, PartialEq)]
pub struct OperatorFunction {
    pub parts: Vec<Box<OperatorArgument>>,

    pub body: Option<Expression>,
    pub return_type: Option<Expression>,
}

#[derive(Eq, PartialEq)]
pub enum OperatorArgument {
    Parameter(Box<Expression>),
    Keyword(String)
}

#[derive(Eq, PartialEq)]
pub struct PatternDeclaration {
    pub precedence: String,

    pub alias: String,
    pub parts: Vec<Box<PatternPart>>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum PatternPart {
    Parameter { key: ParameterKey, internal_name: String },
    Keyword(String),
}

#[derive(Eq, PartialEq)]
pub struct TraitDefinition {
    pub name: String,
    pub statements: Vec<Box<Positioned<Statement>>>,
}

#[derive(Eq, PartialEq)]
pub struct TraitConformanceDeclaration {
    pub declared_for: Expression,
    pub declared: String,
    pub statements: Vec<Box<Positioned<Statement>>>,
}

#[derive(Eq, PartialEq)]
pub struct MemberAccess {
    pub target: Box<Positioned<Term>>,
    pub member: String,
}


// =============================== Code =====================================

#[derive(Eq, PartialEq)]
pub enum Statement {
    VariableDeclaration {
        mutability: Mutability,
        identifier: String,
        type_declaration: Option<Expression>,
        assignment: Option<Expression>
    },
    MemberAssignment { access: MemberAccess, new_value: Expression },
    LocalAssignment { identifier: String, new_value: Expression },
    Expression(Expression),
    Return(Option<Expression>),
    FunctionDeclaration(Box<Function>),
    Operator(Box<OperatorFunction>),
    Pattern(Box<PatternDeclaration>),
    Trait(Box<TraitDefinition>),
    Conformance(Box<TraitConformanceDeclaration>),
}

#[derive(Eq, PartialEq)]
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

#[derive(Eq, PartialEq)]
pub enum Term {
    Error(RuntimeError),
    Identifier(String),
    MacroIdentifier(String),
    IntLiteral(String),
    RealLiteral(String),
    MemberAccess(MemberAccess),
    Struct(Vec<StructArgument>),
    Array(Vec<ArrayArgument>),
    StringLiteral(Vec<Box<Positioned<StringPart>>>),
    Block(Vec<Box<Positioned<Statement>>>),
}

#[derive(Eq, PartialEq)]
pub struct StructArgument {
    pub key: ParameterKey,
    pub value: Expression,
    pub type_declaration: Option<Expression>,
}

#[derive(Eq, PartialEq)]
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

#[derive(PartialEq, Eq)]
pub enum StringPart {
    Literal(String),
    Object(Vec<StructArgument>),
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

impl Display for Module {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        for item in self.global_statements.iter() {
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

impl Display for OperatorFunction {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "def ")?;
        for argument in self.parts.iter() {
            write!(fmt, "{} ", argument)?;
        }
        if let Some(return_type) = &self.return_type {
            write!(fmt, "-> {}", return_type)?;
        }
        if let Some(body) = &self.body {
            write!(fmt, " :: {}", body)?;
        }
        return Ok(())
    }
}

impl Display for FunctionInterface {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionInterface::Macro(m) => write!(fmt, "@{}", m)?,
            FunctionInterface::Explicit { identifier, target_type, parameters, return_type } => {
                if let Some(target_type) = &target_type {
                    write!(fmt, "{{{}}}.", target_type)?;
                }

                write!(fmt, "{}(", identifier)?;
                for item in parameters.iter() { write!(fmt, "{},", item)? };
                write!(fmt, ")")?;

                if let Some(return_type) = &return_type {
                    write!(fmt, " -> {}", return_type)?;
                }
            }
        }
        Ok(())
    }
}

impl Display for PatternDeclaration {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "pattern {}({}) :: ", &self.alias, self.precedence)?;
        write_space_separated_list(fmt, &self.parts)?;
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
            Statement::MemberAssignment { access, new_value } => {
                write!(fmt, "upd {} = {}", access, new_value)
            },
            Statement::LocalAssignment { identifier, new_value } => {
                write!(fmt, "upd {} = {}", identifier, new_value)
            }
            Statement::Return(Some(expression)) => write!(fmt, "return {}", expression),
            Statement::Return(None) => write!(fmt, "return"),
            Statement::Expression(ref expression) => write!(fmt, "{}", expression),

            Statement::FunctionDeclaration(function) => write!(fmt, "{}", function),
            Statement::Pattern(pattern) => write!(fmt, "{}", pattern),
            Statement::Operator(operator) => write!(fmt, "{}", operator),
            Statement::Trait(trait_) => write!(fmt, "{}", trait_),
            Statement::Conformance(conformance) => write!(fmt, "{}", conformance),
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
            Term::MemberAccess(access) =>  write!(fmt, "{}", access),
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
            Term::Block(statements) => {
                write!(fmt, "{{\n")?;
                for item in statements.iter() { write!(fmt, "    {};\n", item)? };
                write!(fmt, "}}")
            }
        }
    }
}

impl Display for MemberAccess {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{}.{}", self.target, self.member)
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

impl Display for PatternPart {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            PatternPart::Parameter { key, internal_name } => write!(fmt, "({}{})", key, internal_name),
            PatternPart::Keyword(keyword) => write!(fmt, "{}", keyword),
        }
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

impl Display for OperatorArgument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatorArgument::Parameter(param) => write!(f, "{{{}}}", param),
            OperatorArgument::Keyword(keyword) => write!(f, "{}", keyword),
        }
    }
}
