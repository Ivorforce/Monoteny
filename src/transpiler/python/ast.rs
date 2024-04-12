use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use itertools::Itertools;

use crate::program::functions::ParameterKey;
use crate::transpiler::python::imperative::escape_string;

pub struct Module {
    // TODO We should use Statement objects instead of 'hardcoding' our structure into this
    //  ast representation.
    //  But before that happens we need to be able to inject comments and have a good indenter.
    pub exported_statements: Vec<Box<Statement>>,
    pub internal_statements: Vec<Box<Statement>>,

    pub exported_names: HashSet<String>,
    pub main_function: Option<String>,
}

impl Display for Module {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "import numpy as np")?;
        writeln!(f, "import math")?;
        writeln!(f, "import operator as op")?;
        writeln!(f, "from dataclasses import dataclass")?;
        writeln!(f, "from numpy import int8, int16, int32, int64, uint8, uint16, uint32, uint64, float32, float64")?;
        writeln!(f, "from typing import Any, Callable")?;
        write!(f, "\n\n")?;

        for statement in self.exported_statements.iter() {
            write!(f, "{}\n\n", statement)?;
        }

        writeln!(f, "# ========================== ======== ============================")?;
        writeln!(f, "# ========================== Internal ============================")?;
        writeln!(f, "# ========================== ======== ============================")?;
        write!(f, "\n\n")?;

        for statement in self.internal_statements.iter() {
            write!(f, "{}\n\n", statement)?;
        }

        writeln!(f, "__all__ = [")?;
        for name in self.exported_names.iter().sorted() {
            writeln!(f, "    \"{}\",", name)?;
        }
        writeln!(f, "]")?;

        if let Some(main_function) = &self.main_function {
            write!(f, "\n\nif __name__ == \"__main__\":\n    {}()\n", main_function)?;
        }

        Ok(())
    }
}

pub struct Class {
    pub name: String,
    pub statements: Vec<Box<Statement>>,
}

impl Display for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "@dataclass\nclass {}:\n", self.name)?;
        if self.statements.is_empty() {
            writeln!(f, "    pass")?;
        }
        else {
            for statement in self.statements.iter() {
                write!(f, "    {}", statement)?;
            }
        }
        Ok(())
    }
}

pub struct Function {
    pub name: String,

    pub parameters: Vec<Box<Parameter>>,
    pub return_type: Option<Box<Expression>>,
    pub statements: Vec<Box<Statement>>,
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "def {}(", self.name)?;
        for (idx, parameter) in self.parameters.iter().enumerate() {
            write!(f, "{}", parameter)?;

            if idx < self.parameters.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")?;

        if let Some(return_type) = &self.return_type {
            write!(f, " -> {}", return_type)?;
        }

        write!(f, ":\n")?;

        write!(f, "    \"\"\"\n    <DOCSTRING TODO>")?;

        if !self.parameters.is_empty() {
            write!(f, "\n\n    Args:")?;

            for (idx, parameter) in self.parameters.iter().enumerate() {
                write!(f, "\n        {}: TODO", parameter.name)?;
            }
        }

        if self.return_type.is_some() {
            write!(f, "\n\n    Returns:\n")?;
            write!(f, "        <TODO>")?;
        }

        write!(f, "\n    \"\"\"\n")?;

        if self.statements.is_empty() {
            writeln!(f, "pass")?;
            return Ok(());
        }

        for statement in self.statements.iter() {
            write!(f, "    {}", statement)?;
        }

        Ok(())
    }
}

pub enum Statement {
    VariableAssignment { target: Box<Expression>, value: Option<Box<Expression>>, type_annotation: Option<Box<Expression>> },
    Expression(Box<Expression>),
    Return(Option<Box<Expression>>),
    Class(Box<Class>),
    Function(Box<Function>),
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::VariableAssignment { target: variable_name, value, type_annotation } => {
                write!(f, "{}", variable_name)?;

                if let Some(type_annotation) = type_annotation {
                    write!(f, ": {}", type_annotation)?;
                }
                if let Some(value) = value {
                    write!(f, " = {}", value)?;
                }

                writeln!(f)?;

                Ok(())
            }
            Statement::Expression(e) => writeln!(f, "{}", e),
            Statement::Return(Some(expression)) => {
                writeln!(f, "return {}", expression)
            }
            Statement::Return(None) => {
                writeln!(f, "return")
            }
            Statement::Class(c) => write!(f, "{}", c),
            Statement::Function(fun) => write!(f, "{}", fun),
        }
    }
}

pub enum Expression {
    MemberAccess(Box<Expression>, String),
    UnaryOperation(String, Box<Expression>),
    BinaryOperation(Box<Expression>, String, Box<Expression>),
    FunctionCall(Box<Expression>, Vec<(ParameterKey, Box<Expression>)>),
    NamedReference(String),
    StringLiteral(String),
    ValueLiteral(String),
}

impl Expression {
    pub fn is_simple(&self) -> bool {
        match self {
            Expression::UnaryOperation(_, _) => false,
            Expression::BinaryOperation(_, _, _) => false,
            Expression::FunctionCall(_, _) => true,
            Expression::NamedReference(_) => true,
            Expression::StringLiteral(_) => true,
            Expression::ValueLiteral(_) => true,
            Expression::MemberAccess(_, _) => true,
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::UnaryOperation(op, ex) => {
                if matches!(op.as_str(), "+" | "-") {
                    write!(f, "{}", op)?;
                }
                else {
                    write!(f, "{} ", op)?;
                }
                write_maybe_paranthesized(f, ex, !ex.is_simple())
            }
            Expression::BinaryOperation(lhs, op, rhs) => {
                write_maybe_paranthesized(f, lhs, !lhs.is_simple())?;
                write!(f, " {} ", op)?;
                write_maybe_paranthesized(f, rhs, !rhs.is_simple())
            }
            Expression::FunctionCall(name, params) => {
                write!(f, "{}(", name)?;

                for (i, (key, argument)) in params.iter().enumerate() {
                    if let ParameterKey::Name(name) = key {
                        write!(f, "{}=", name)?;
                    }
                    write!(f, "{}", argument)?;

                    if i < params.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, ")")
            }
            Expression::NamedReference(v) => {
                write!(f, "{}", v)
            }
            Expression::StringLiteral(v) => {
                write!(f, "\"{}\"", escape_string(v))
            }
            Expression::ValueLiteral(v) => {
                write!(f, "{}", v)
            }
            Expression::MemberAccess(e, m) => {
                write!(f, "{}.{}", e, m)
            }
        }
    }
}

pub struct Parameter {
    pub name: String,
    pub type_: Box<Expression>,
}

impl Display for Parameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.type_)
    }
}

pub fn write_maybe_paranthesized<D: Display>(f: &mut Formatter, d: D, parenthesize: bool) -> std::fmt::Result {
    if parenthesize {
        write!(f, "({})", d)
    }
    else {
        write!(f, "{}", d)
    }
}
