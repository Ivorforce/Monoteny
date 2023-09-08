use std::fmt::{Display, Formatter};
use itertools::Itertools;
use crate::program::functions::ParameterKey;
use crate::transpiler::python::imperative::escape_string;

pub struct Module {
    pub exported_classes: Vec<Box<Class>>,
    pub exported_functions: Vec<Box<Function>>,
    pub internal_functions: Vec<Box<Function>>,

    pub main_function: Option<String>,
}

impl Display for Module {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "import numpy as np")?;
        writeln!(f, "import math")?;
        writeln!(f, "import operator as op")?;
        writeln!(f, "from numpy import int8, int16, int32, int64, uint8, uint16, uint32, uint64, float32, float64")?;
        writeln!(f, "from typing import Any, Callable")?;
        write!(f, "\n\n")?;

        for struct_ in self.exported_classes.iter() {
            write!(f, "{}\n\n", struct_)?;
        }

        for function in self.exported_functions.iter() {
            write!(f, "{}\n\n", function)?;
        }

        writeln!(f, "# ========================== ======== ============================")?;
        writeln!(f, "# ========================== Internal ============================")?;
        writeln!(f, "# ========================== ======== ============================")?;
        write!(f, "\n\n")?;

        for function in self.internal_functions.iter() {
            write!(f, "{}\n\n", function)?;
        }

        writeln!(f, "__all__ = [")?;
        for name in self.exported_functions.iter().map(|x| &x.name)
            .chain(self.exported_classes.iter().map(|x| &x.name)).sorted() {
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
}

impl Display for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n\nclass {}:\n    pass", self.name)
    }
}

pub struct Function {
    pub name: String,

    pub parameters: Vec<Box<Parameter>>,
    pub return_type: Option<String>,
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
            write!(f, "pass\n")?;
            return Ok(());
        }

        for statement in self.statements.iter() {
            writeln!(f, "    {}", statement)?;
        }

        Ok(())
    }
}

pub enum Statement {
    VariableAssignment { variable_name: String, value: Box<Expression> },
    Expression(Box<Expression>),
    Return(Option<Box<Expression>>),
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::VariableAssignment { variable_name, value } => {
                write!(f, "{} = {}", variable_name, value)
            }
            Statement::Expression(e) => e.fmt(f),
            Statement::Return(Some(expression)) => {
                write!(f, "return {}", expression)
            }
            Statement::Return(None) => {
                write!(f, "return")
            }
        }
    }
}

pub enum Expression {
    UnaryOperation(String, Box<Expression>),
    BinaryOperation(Box<Expression>, String, Box<Expression>),
    FunctionCall(String, Vec<(ParameterKey, Box<Expression>)>),
    VariableLookup(String),
    StringLiteral(String),
    ValueLiteral(String),
}

impl Expression {
    pub fn is_simple(&self) -> bool {
        match self {
            Expression::UnaryOperation(_, _) => false,
            Expression::BinaryOperation(_, _, _) => false,
            Expression::FunctionCall(_, _) => true,
            Expression::VariableLookup(_) => true,
            Expression::StringLiteral(_) => true,
            Expression::ValueLiteral(_) => true,
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::UnaryOperation(op, ex) => {
                write!(f, "{}", op)?;
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
            Expression::VariableLookup(v) => {
                write!(f, "{}", v)
            }
            Expression::StringLiteral(v) => {
                write!(f, "\"{}\"", escape_string(v))
            }
            Expression::ValueLiteral(v) => {
                write!(f, "{}", v)
            }
        }
    }
}

pub struct Parameter {
    pub name: String,
    pub type_: String,
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
