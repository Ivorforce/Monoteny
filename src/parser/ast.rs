use std::fmt::{Display, Error, Formatter};
use std::ops::{Deref, DerefMut};
use itertools::Itertools;
use crate::error::RuntimeError;
use crate::program::functions::ParameterKey;
use crate::program::allocation::Mutability;
use crate::program::types::PatternPart;
use crate::util::fmt::{write_comma_separated_list, write_space_separated_list};
use crate::util::position::Positioned;

// =============================== Global =====================================

#[derive(Eq, PartialEq)]
pub struct Module {
    pub global_statements: Vec<Box<Positioned<GlobalStatement>>>
}

#[derive(Eq, PartialEq)]
pub enum GlobalStatement {
    FunctionDeclaration(Box<Function>),
    Operator(Box<OperatorFunction>),
    Pattern(Box<PatternDeclaration>),
    Trait(Box<TraitDefinition>),
    Conformance(Box<TraitConformanceDeclaration>),
    Macro(Box<GlobalMacro>),
    Error(RuntimeError),
}

#[derive(Eq, PartialEq)]
pub enum MemberStatement {
    FunctionDeclaration(Box<Function>),
}

#[derive(Eq, PartialEq)]
pub struct Function {
    pub decorators: Vec<String>,

    pub target_type: Option<Box<Expression>>,
    pub identifier: String,
    pub parameters: Vec<Box<KeyedParameter>>,
    pub return_type: Option<Expression>,

    pub body: Option<Expression>,
}

#[derive(Eq, PartialEq)]
pub struct GlobalMacro {
    pub decorators: Vec<String>,
    pub macro_name: String,
    pub body: Option<Expression>,
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
    pub decorators: Vec<String>,

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

#[derive(Eq, PartialEq)]
pub struct TraitDefinition {
    pub decorators: Vec<String>,

    pub name: String,
    pub statements: Vec<Box<Positioned<GlobalStatement>>>,
}

#[derive(Eq, PartialEq)]
pub struct TraitConformanceDeclaration {
    pub declared_for: Expression,
    pub declared: String,
    pub statements: Vec<Box<Positioned<GlobalStatement>>>,
}


// =============================== Code =====================================

#[derive(Eq, PartialEq)]
pub enum Statement {
    VariableDeclaration {
        mutability: Mutability,
        identifier: String,
        type_declaration: Option<Expression>,
        expression: Expression
    },
    VariableAssignment { variable_name: String, new_value: Expression },
    Expression(Expression),
    Return(Option<Expression>),
    Error(RuntimeError),
}

#[derive(Eq, PartialEq)]
pub struct Expression(Vec<Box<Positioned<Term>>>);

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
    Identifier(String),
    IntLiteral(String),
    RealLiteral(String),
    MemberAccess { target: Box<Positioned<Term>>, member_name: String },
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

impl Display for GlobalStatement {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            GlobalStatement::FunctionDeclaration(function) => write!(fmt, "{}", function),
            GlobalStatement::Pattern(pattern) => write!(fmt, "{}", pattern),
            GlobalStatement::Operator(operator) => write!(fmt, "{}", operator),
            GlobalStatement::Trait(trait_) => write!(fmt, "{}", trait_),
            GlobalStatement::Conformance(conformance) => write!(fmt, "{}", conformance),
            GlobalStatement::Macro(macro_) => write!(fmt, "{}", macro_),
            GlobalStatement::Error(string) => write!(fmt, "{}", string),
        }?;

        write!(fmt, ";")
    }
}

impl Display for MemberStatement {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            MemberStatement::FunctionDeclaration(function) => write!(fmt, "{}", function),
        }
    }
}

impl Display for Function {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        for decorator in self.decorators.iter() {
            writeln!(fmt, "@{}", decorator)?;
        }
        write!(fmt, "def ")?;
        if let Some(target_type) = &self.target_type {
            write!(fmt, "{{{}}}.", target_type)?;
        }
        write!(fmt, "{}(", self.identifier)?;
        for item in self.parameters.iter() { write!(fmt, "{},", item)? };
        write!(fmt, ")")?;
        if let Some(return_type) = &self.return_type {
            write!(fmt, " -> {}", return_type)?;
        }
        if let Some(body) = &self.body {
            write!(fmt, " :: {}", body)?;
        }
        return Ok(())
    }
}

impl Display for OperatorFunction {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        for decorator in self.decorators.iter() {
            writeln!(fmt, "@{}", decorator)?;
        }
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

impl Display for GlobalMacro {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        for decorator in self.decorators.iter() {
            writeln!(fmt, "@{}", decorator)?;
        }
        write!(fmt, "def @{}", self.macro_name)?;
        if let Some(body) = &self.body {
            write!(fmt, " :: {}", body)?;
        }
        return Ok(())
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
            Statement::VariableDeclaration { mutability, identifier, type_declaration, expression} => {
                let mutability_string = mutability.variable_declaration_keyword();
                write!(fmt, "{} {}", mutability_string, identifier)?;
                if let Some(type_declaration) = type_declaration {
                    write!(fmt, " '{}", type_declaration)?;
                }
                write!(fmt, " = {}", expression)
            },
            Statement::VariableAssignment { variable_name, new_value } => {
                write!(fmt, "upd {} = {}", variable_name, new_value)
            },
            Statement::Return(Some(expression)) => write!(fmt, "return {}", expression),
            Statement::Return(None) => write!(fmt, "return"),
            Statement::Expression(ref expression) => write!(fmt, "{}", expression),
            Statement::Error(string) => write!(fmt, "{}", string),
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
            Term::Identifier(s) => write!(fmt, "{}", s),
            Term::IntLiteral(s) => write!(fmt, "{}", s),
            Term::RealLiteral(s) => write!(fmt, "{}", s),
            Term::StringLiteral(parts) => {
                write!(fmt, "\"")?;
                for part in parts {
                    write!(fmt, "{}", part)?;
                }
                write!(fmt, "\"")
            },
            Term::MemberAccess { target, member_name } =>  write!(fmt, "{}.{}", target.value, member_name),
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
