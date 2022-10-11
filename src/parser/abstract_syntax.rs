use std::fmt::{Binary, Debug, Error, Formatter};
use itertools::zip_eq;
use crate::program::functions::ParameterKey;
use crate::program::allocation::Mutability;
use crate::util::fmt::write_comma_separated_list;

// =============================== Global =====================================

pub struct Program {
    pub global_statements: Vec<Box<GlobalStatement>>
}

pub enum GlobalStatement {
    FunctionDeclaration(Box<Function>),
    Operator(Box<Operator>),
    Pattern(Box<PatternDeclaration>),
}

pub struct Function {
    pub target: Option<Box<ContextualParameter>>,
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

pub struct Operator {
    pub lhs: Option<Box<ContextualParameter>>,
    pub operator: String,
    pub rhs: Box<ContextualParameter>,

    pub body: Vec<Box<Statement>>,
    pub return_type: Option<Expression>,
}

pub struct ContextualParameter {
    pub internal_name: String,
    pub param_type: Expression,
}

pub enum MemberStatement {
    FunctionDeclaration(Box<Function>),
}

pub struct PatternDeclaration {
    pub form: PatternForm,

    pub operator: String,
    pub precedence: String,

    pub alias: String,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PatternForm {
    Unary, Binary
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
        if let Some(target_type) = &self.target {
            write!(fmt, "{:?}.", target_type)?;
        }
        write!(fmt, "{}(", self.identifier)?;
        for item in self.parameters.iter() { write!(fmt, "{:?},", item)? };
        write!(fmt, ") -> {:?} {{\n", self.return_type)?;
        for item in self.body.iter() { write!(fmt, "    {:?};\n", item)? };
        write!(fmt, "}}")?;
        return Ok(())
    }
}

impl Debug for Operator {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "fun ")?;
        if let Some(lhs) = &self.lhs {
            write!(fmt, "{:?} ", lhs)?;
        }
        write!(fmt, "{}", self.operator)?;
        write!(fmt, " -> {:?} {{\n", self.return_type)?;
        for item in self.body.iter() { write!(fmt, "    {:?};\n", item)? };
        write!(fmt, "}}")?;
        return Ok(())
    }
}

impl Debug for PatternDeclaration {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "// alias: {}\n", &self.alias)?;

        match &self.form {
            PatternForm::Unary => write!(fmt, "pattern {{}} {:?} {{}} :: {}", &self.operator, &self.precedence),
            PatternForm::Binary => write!(fmt, "pattern {{}} {:?} :: {}", &self.operator, &self.precedence)
        }
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

impl Debug for KeyedParameter {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?} {}: {:?}", self.key, self.internal_name, self.param_type)
    }
}

impl Debug for ContextualParameter {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{{{}: {:?}}}", self.internal_name, self.param_type)
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
