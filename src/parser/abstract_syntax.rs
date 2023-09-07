use std::fmt::{Binary, Debug, Error, Formatter};
use crate::program::functions::ParameterKey;
use crate::program::allocation::Mutability;
use crate::program::types::PatternPart;
use crate::util::fmt::{write_comma_separated_list, write_space_separated_list};

// =============================== Global =====================================

#[derive(Eq, PartialEq)]
pub struct Program {
    pub global_statements: Vec<Box<GlobalStatement>>
}

#[derive(Eq, PartialEq)]
pub enum GlobalStatement {
    FunctionDeclaration(Box<Function>),
    Operator(Box<OperatorFunction>),
    Pattern(Box<PatternDeclaration>),
    Trait(Box<TraitDefinition>),
    Conformance(Box<TraitConformanceDeclaration>),
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
    pub statements: Vec<Box<GlobalStatement>>,
}

#[derive(Eq, PartialEq)]
pub struct TraitConformanceDeclaration {
    pub target: String,
    pub trait_: String,
    pub statements: Vec<Box<GlobalStatement>>,
}


// =============================== Code =====================================

pub type Expression = Vec<Box<Term>>;

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
}

#[derive(Eq, PartialEq)]
pub enum Term {
    Identifier(String),
    Int(String),
    Float(String),
    MemberAccess { target: Box<Term>, member_name: String },
    Struct(Vec<StructArgument>),
    Array(Vec<ArrayArgument>),
    StringLiteral(String),
    Scope(Vec<Box<Statement>>),
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

#[derive(Copy, Clone, PartialEq)]
pub enum FunctionCallType {
    Call,
    Subscript,
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

impl Debug for Program {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        for item in self.global_statements.iter() {
            write!(fmt, "{:?}\n\n", item)?
        };
        return Ok(())
    }
}

impl Debug for GlobalStatement {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            GlobalStatement::FunctionDeclaration(function) => write!(fmt, "{:?}", function),
            GlobalStatement::Pattern(pattern) => write!(fmt, "{:?}", pattern),
            GlobalStatement::Operator(operator) => write!(fmt, "{:?}", operator),
            GlobalStatement::Trait(trait_) => write!(fmt, "{:?}", trait_),
            GlobalStatement::Conformance(conformance) => write!(fmt, "{:?}", conformance),
        }
    }
}

impl Debug for MemberStatement {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            MemberStatement::FunctionDeclaration(function) => write!(fmt, "{:?}", function),
        }
    }
}

impl Debug for Function {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "def ")?;
        if let Some(target_type) = &self.target_type {
            write!(fmt, "{{{:?}}}.", target_type)?;
        }
        write!(fmt, "{}(", self.identifier)?;
        for item in self.parameters.iter() { write!(fmt, "{:?},", item)? };
        write!(fmt, ") -> {:?} {{\n", self.return_type)?;
        for item in self.body.iter() { write!(fmt, "    {:?};\n", item)? };
        write!(fmt, "}}")?;
        return Ok(())
    }
}

impl Debug for OperatorFunction {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "def ")?;
        for argument in self.parts.iter() {
            match argument.as_ref() {
                OperatorArgument::Parameter(param) => write!(fmt, "{{{:?}}} ", param)?,
                OperatorArgument::Keyword(keyword) => write!(fmt, "{:?} ", keyword)?,
            }
        }
        write!(fmt, " -> {:?} {{\n", self.return_type)?;
        write!(fmt, "{:?}", self.body)?;
        write!(fmt, "}}")?;
        return Ok(())
    }
}

impl Debug for PatternDeclaration {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "pattern {}({}) :: ", &self.alias, self.precedence)?;
        write_space_separated_list(fmt, &self.parts)?;
        write!(fmt, ";")?;
        Ok(())
    }
}

impl Debug for TraitDefinition {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "trait {} {{", self.name)?;

        Ok(())
    }
}

impl Debug for TraitConformanceDeclaration {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "declare {} is {} {{}} :: ", self.target, self.trait_)?;
        Ok(())
    }
}

impl Debug for Statement {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Statement::VariableDeclaration { mutability, identifier, type_declaration, expression} => {
                let mutability_string = mutability.variable_declaration_keyword();
                write!(fmt, "{} {}: {:?} = {:?}", mutability_string, identifier, type_declaration, expression)
            },
            Statement::VariableAssignment { variable_name, new_value } => {
                write!(fmt, "{} = {:?}", variable_name, new_value)
            },
            Statement::Return(ref expression) => write!(fmt, "return {:?}", expression),
            Statement::Expression(ref expression) => write!(fmt, "{:?}", expression),
        }
    }
}

impl Debug for Term {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Term::Identifier(s) => write!(fmt, "{}", s),
            Term::Int(s) => write!(fmt, "{}", s),
            Term::Float(s) => write!(fmt, "{}", s),
            Term::StringLiteral(string) => write!(fmt, "{:?}", string),
            Term::MemberAccess { target, member_name } =>  write!(fmt, "{:?}.{}", target, member_name),
            Term::Struct(arguments) => {
                write!(fmt, "(")?;
                write_comma_separated_list(fmt, arguments)?;
                write!(fmt, ")")?;
                return Ok(())
            },
            Term::Array(arguments) => {
                write!(fmt, "[")?;
                write_comma_separated_list(fmt, arguments)?;
                write!(fmt, "]")?;
                return Ok(())
            },
            Term::Scope(statements) => {
                write!(fmt, "{{\n")?;
                for item in statements.iter() { write!(fmt, "    {:?};\n", item)? };
                write!(fmt, "}}")
            }
        }
    }
}

impl Debug for StructArgument {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?}: {:?}", self.key, self.value)
    }
}

impl Debug for ArrayArgument {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?}: {:?}", self.key, self.value)
    }
}

impl Debug for PatternPart {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            PatternPart::Parameter { key, internal_name } => write!(fmt, "({:?}:{})", key, internal_name),
            PatternPart::Keyword(keyword) => write!(fmt, "{}", keyword),
        }
    }
}

impl Debug for KeyedParameter {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?} {}: {:?}", self.key, self.internal_name, self.param_type)
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
