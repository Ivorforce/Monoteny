use std::fmt::{Binary, Debug, Error, Formatter};
use std::iter::zip;
use crate::program::types::{Mutability, ParameterKey};

// =============================== Global =====================================

pub struct Program {
    pub global_statements: Vec<Box<GlobalStatement>>
}

pub enum GlobalStatement {
    FunctionDeclaration(Box<Function>),
}

pub struct Function {
    pub target_type: Option<Box<TypeDeclaration>>,
    pub identifier: String,
    pub parameters: Vec<Box<Parameter>>,
    pub return_type: Option<Box<TypeDeclaration>>,
    pub body: Vec<Box<Statement>>
}

pub struct Parameter {
    pub key: ParameterKey,
    pub internal_name: String,
    pub param_type: Box<TypeDeclaration>,
}

pub enum MemberStatement {
    FunctionDeclaration(Box<Function>),
}

// =============================== Type =====================================

pub enum TypeDeclaration {
    Identifier(String),
    NDArray(Box<TypeDeclaration>, Vec<Box<Expression>>)
}

// =============================== Code =====================================

pub enum Statement {
    VariableDeclaration {
        mutability: Mutability,
        identifier: String,
        type_declaration: Option<Box<TypeDeclaration>>,
        expression: Box<Expression>
    },
    VariableAssignment { variable_name: String, new_value: Box<Expression> },
    Expression(Box<Expression>),
    Return(Option<Box<Expression>>),
}

pub enum Expression {
    Number(i32),
    Bool(bool),
    BinaryOperator { lhs: Box<Expression>, operator: String, rhs: Box<Expression> },
    UnaryOperator { operator: String, argument: Box<Expression> },
    ConjunctivePairOperators { arguments: Vec<Box<Expression>>, operators: Vec<String> },
    UnsortedBinaryOperators { arguments: Vec<Box<Expression>>, operators: Vec<String> },
    FunctionCall { call_type: FunctionCallType, callee: Box<Expression>, arguments: Vec<Box<PassedArgument>> },
    MemberLookup { target: Box<Expression>, member_name: String },
    VariableLookup(String),
    ArrayLiteral(Vec<Box<Expression>>),
    StringLiteral(String),
}

pub struct PassedArgument {
    pub key: Option<ParameterKey>,
    pub value: Box<Expression>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum FunctionCallType {
    Call,
    Subscript,
}

// =============================== String =====================================

fn is_simple(expression: &Expression) -> bool {
    match expression {
        Expression::Number(_) => true,
        Expression::Bool(_) => true,
        Expression::BinaryOperator { .. } => false,
        Expression::UnsortedBinaryOperators { .. } => false,
        Expression::ConjunctivePairOperators { .. } => false,
        Expression::UnaryOperator { .. } => true,
        Expression::FunctionCall { .. } => true,
        Expression::MemberLookup { .. } => true,
        Expression::VariableLookup(_) => true,
        Expression::ArrayLiteral(_) => true,
        Expression::StringLiteral(_) => true,
    }
}

impl Mutability {
    fn variable_declaration_keyword(&self) -> &str {
        return match *self {
            Mutable => "var",
            Immutable => "let",
        };
    }
}

fn write_maybe_parenthesized_expression(fmt: &mut Formatter, expression: &Expression) -> Result<(), Error> {
    if is_simple(expression) {
        write!(fmt, "{:?}", expression)
    }
    else {
        write!(fmt, "({:?})", expression)
    }
}

fn write_comma_separated_list<E>(fmt: &mut Formatter, list: &Vec<E>) -> Result<(), Error> where E: Debug {
    if let Some(first) = list.first() {
        write!(fmt, "{:?}", first)?
    }
    for item in list.iter().skip(1) { write!(fmt, ", {:?}", item)? }
    Ok(())
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
        write!(fmt, "fn ")?;
        if let Some(target_type) = &self.target_type {
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

impl Debug for TypeDeclaration {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::TypeDeclaration::*;
        match self {
            Identifier(name) => write!(fmt, "{}", name),
            NDArray(atom, dimensions) => {
                write!(fmt, "{:?}[", atom)?;
                for item in dimensions { write!(fmt, "{:?},", item)? };
                write!(fmt, "]")?;
                return Ok(())
            },
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

impl Debug for Expression {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Expression::*;
        match self {
            Number(n) => write!(fmt, "{:?}", n),
            BinaryOperator { lhs, operator, rhs } => {
                write_maybe_parenthesized_expression(fmt, lhs.as_ref())?;
                write!(fmt, " {:?} ", operator)?;
                write_maybe_parenthesized_expression(fmt, rhs.as_ref())?;
                return Ok(())
            },
            UnaryOperator { operator, argument } => write!(fmt, "{:?}{:?}", operator, argument),
            UnsortedBinaryOperators { .. } => {
                panic!("Cannot debug at this stage; please run parser fully before printing.")
            }
            ConjunctivePairOperators { arguments, operators } => {
                for (argument, operator) in zip(arguments, operators) {
                    write_maybe_parenthesized_expression(fmt, argument.as_ref())?;
                    write!(fmt, " {:?} ", operator)?;
                }
                write_maybe_parenthesized_expression(fmt, arguments.last().unwrap().as_ref())?;
                return Ok(())
            },
            FunctionCall { call_type, callee, arguments } => {
                let brackets = call_type.bracket_str();
                write!(fmt, "{:?}{}", callee, brackets.chars().nth(0).unwrap())?;
                write_comma_separated_list(fmt, arguments)?;
                write!(fmt, "{}", brackets.chars().nth(1).unwrap())?;
                return Ok(())
            },
            VariableLookup(id) => write!(fmt, "{}", id),
            MemberLookup { target, member_name } => write!(fmt, "{:?}.{}", target, member_name),
            ArrayLiteral(items) => {
                write!(fmt, "[")?;
                write_comma_separated_list(fmt, items)?;
                write!(fmt, "]")?;
                return Ok(())
            },
            StringLiteral(string) => write!(fmt, "{:?}", string),
            Bool(value) => write!(fmt, "{}", value),
        }
    }
}

impl Debug for PassedArgument {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?}: {:?}", self.key, self.value)
    }
}

impl Debug for Parameter {
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
