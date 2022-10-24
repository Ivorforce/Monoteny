use std::fmt::{Binary, Debug, Error, Formatter};
use itertools::zip_eq;
use crate::program::functions::ParameterKey;
use crate::program::allocation::Mutability;
use crate::program::types::PatternPart;
use crate::util::fmt::{write_comma_separated_list, write_space_separated_list};

// =============================== Global =====================================

pub struct Program {
    pub global_statements: Vec<Box<GlobalStatement>>
}

pub enum GlobalStatement {
    FunctionDeclaration(Box<Function>),
    Operator(Box<OperatorFunction>),
    Pattern(Box<PatternDeclaration>),
    Constant(Box<Constant>),
}

pub enum MemberStatement {
    FunctionDeclaration(Box<Function>),
}

pub struct Function {
    pub target_type: Option<Box<Expression>>,
    pub identifier: String,
    pub parameters: Vec<Box<KeyedParameter>>,

    pub body: Vec<Box<Statement>>,
    pub return_type: Option<Expression>,
}

pub struct KeyedParameter {
    pub key: ParameterKey,
    pub internal_name: String,
    pub param_type: Expression,
}

pub struct OperatorFunction {
    pub parts: Vec<Box<OperatorArgument>>,

    pub body: Vec<Box<Statement>>,
    pub return_type: Option<Expression>,
}

pub enum OperatorArgument {
    Parameter(Box<Expression>),
    Keyword(String)
}

pub struct PatternDeclaration {
    pub precedence: String,

    pub alias: String,
    pub parts: Vec<Box<PatternPart>>,
}

pub struct Constant {
    pub identifier: String,

    pub body: Vec<Box<Statement>>,
    pub declared_type: Expression,
}

// =============================== Code =====================================

pub type Expression = Vec<Box<Term>>;

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

pub enum Term {
    Identifier(String),
    Number(String),
    MemberAccess { target: Box<Term>, member_name: String },
    TypeHint { object: Box<Term>, type_: Box<Term> },
    Struct(Vec<StructArgument>),
    Array(Vec<ArrayArgument>),
    StringLiteral(String),
}

pub struct StructArgument {
    pub key: ParameterKey,
    pub value: Expression,
}

pub struct ArrayArgument {
    pub key: Option<Expression>,
    pub value: Expression,
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
            Mutable => "var",
            Immutable => "let",
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
        use self::GlobalStatement::*;
        match self {
            FunctionDeclaration(function) => write!(fmt, "{:?}", function),
            Pattern(pattern) => write!(fmt, "{:?}", pattern),
            Operator(operator) => write!(fmt, "{:?}", operator),
            Constant(constant) => write!(fmt, "{:?}", constant),
        }
    }
}

impl Debug for MemberStatement {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::MemberStatement::*;
        match self {
            FunctionDeclaration(function) => write!(fmt, "{:?}", function),
        }
    }
}

impl Debug for Function {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "fun ")?;
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
        write!(fmt, "fun ")?;
        for argument in self.parts.iter() {
            match argument.as_ref() {
                OperatorArgument::Parameter(param) => write!(fmt, "{{{:?}}} ", param)?,
                OperatorArgument::Keyword(keyword) => write!(fmt, "{:?} ", keyword)?,
            }
        }
        write!(fmt, " -> {:?} {{\n", self.return_type)?;
        for item in self.body.iter() { write!(fmt, "    {:?};\n", item)? };
        write!(fmt, "}}")?;
        return Ok(())
    }
}

impl Debug for Constant {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "let {} '", self.identifier)?;
        for item in self.declared_type.iter() { write!(fmt, "{:?} ", item)? };
        write!(fmt, ":: {{")?;
        for item in self.body.iter() { write!(fmt, "    {:?};\n", item)? };
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

impl Debug for Statement {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Statement::*;
        match self {
            VariableDeclaration { mutability, identifier, type_declaration, expression} => {
                let mutability_string = mutability.variable_declaration_keyword();
                write!(fmt, "{} {}: {:?} = {:?}", mutability_string, identifier, type_declaration, expression)
            },
            VariableAssignment { variable_name, new_value } => {
                write!(fmt, "{} = {:?}", variable_name, new_value)
            },
            Return(ref expression) => write!(fmt, "return {:?}", expression),
            Expression(ref expression) => write!(fmt, "{:?}", expression),
        }
    }
}

impl Debug for Term {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Term::*;
        match self {
            Identifier(s) => write!(fmt, "{}", s),
            Number(s) => write!(fmt, "{}", s),
            StringLiteral(string) => write!(fmt, "{:?}", string),
            MemberAccess { target, member_name } =>  write!(fmt, "{:?}.{}", target, member_name),
            Struct(arguments) => {
                write!(fmt, "(")?;
                write_comma_separated_list(fmt, arguments)?;
                write!(fmt, ")")?;
                return Ok(())
            },
            Array(arguments) => {
                write!(fmt, "[")?;
                write_comma_separated_list(fmt, arguments)?;
                write!(fmt, "]")?;
                return Ok(())
            },
            TypeHint { object, type_ } => write!(fmt, "{:?} '{:?}", object, type_),
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
        use self::FunctionCallType::*;
        return match *self {
            Call => "()",
            Subscript => "[]",
        };
    }
}
