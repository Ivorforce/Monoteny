use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::fmt::Write;

use itertools::Itertools;
use display_with_options::{DisplayWithOptions, IndentingFormatter, IndentOptions, with_options};

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

impl<'a> DisplayWithOptions<IndentOptions<'a>> for Module {
    fn fmt(&self, f: &mut Formatter, options: &IndentOptions) -> std::fmt::Result {
        let mut f = IndentingFormatter::new(f, &options.full_indentation);

        writeln!(f, "import numpy as np")?;
        writeln!(f, "import math")?;
        writeln!(f, "import operator as op")?;
        writeln!(f, "from dataclasses import dataclass")?;
        writeln!(f, "from numpy import int8, int16, int32, int64, uint8, uint16, uint32, uint64, float32, float64")?;
        writeln!(f, "from typing import Any, Callable")?;
        write!(f, "\n\n")?;

        for statement in self.exported_statements.iter() {
            write!(f, "{}\n\n", with_options(statement.as_ref(), &options.restart()))?;
        }

        writeln!(f, "# ========================== ======== ============================")?;
        writeln!(f, "# ========================== Internal ============================")?;
        writeln!(f, "# ========================== ======== ============================")?;
        write!(f, "\n\n")?;

        for statement in self.internal_statements.iter() {
            write!(f, "{}\n\n", with_options(statement.as_ref(), &options.restart()))?;
        }

        writeln!(f, "__all__ = [")?;
        for name in self.exported_names.iter().sorted() {
            writeln!(f, "{}\"{}\",", options.next_level, name)?;
        }
        writeln!(f, "]")?;

        if let Some(main_function) = &self.main_function {
            write!(f, "\n\nif __name__ == \"__main__\":\n{}{}()\n", options.next_level, main_function)?;
        }

        Ok(())
    }
}

pub struct Class {
    pub name: String,
    pub block: Block,
}

impl<'a> DisplayWithOptions<IndentOptions<'a>> for Class {
    fn fmt(&self, f: &mut Formatter, options: &IndentOptions) -> std::fmt::Result {
        write!(f, "{}@dataclass\nclass {}:\n", options, self.name)?;

        let options = options.deeper();
        let mut f = IndentingFormatter::new(f, &options.full_indentation);
        let options = options.restart();

        write!(f, "{}", with_options(&self.block, &options))?;

        Ok(())
    }
}

pub struct Block {
    pub statements: Vec<Box<Statement>>,
}

impl<'a> DisplayWithOptions<IndentOptions<'a>> for Block {
    fn fmt(&self, f: &mut Formatter, options: &IndentOptions) -> std::fmt::Result {
        if self.statements.is_empty() {
            writeln!(f, "{}pass", options)?;
        }
        else {
            for statement in self.statements.iter() {
                write!(f, "{}", with_options(statement.as_ref(), options))?;
            }
        }
        Ok(())
    }
}

pub struct Function {
    pub name: String,

    pub parameters: Vec<Box<Parameter>>,
    pub return_type: Option<Box<Expression>>,
    pub block: Box<Block>,
}

impl<'a> DisplayWithOptions<IndentOptions<'a>> for Function {
    fn fmt(&self, f: &mut Formatter, options: &IndentOptions) -> std::fmt::Result {
        write!(f, "{}def {}(", options, self.name)?;
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

        let options = options.deeper();
        let mut f = IndentingFormatter::new(f, &options.full_indentation);
        let options = options.restart();
        let indent_once = options.deeper();

        write!(f, "\"\"\"\n<DOCSTRING TODO>")?;

        if !self.parameters.is_empty() {
            write!(f, "\n\n{}Args:", options)?;

            for (idx, parameter) in self.parameters.iter().enumerate() {
                write!(f, "\n{}{}: <TODO>", indent_once, parameter.name)?;
            }
        }

        if self.return_type.is_some() {
            write!(f, "\n\nReturns:\n")?;
            write!(f, "{}<TODO>", indent_once)?;
        }

        write!(f, "\n\"\"\"\n")?;

        write!(f, "{}", with_options(self.block.as_ref(), &options))?;

        Ok(())
    }
}

pub enum Statement {
    VariableAssignment { target: Box<Expression>, value: Option<Box<Expression>>, type_annotation: Option<Box<Expression>> },
    Expression(Box<Expression>),
    Return(Option<Box<Expression>>),
    Class(Box<Class>),
    Function(Box<Function>),
    IfThenElse(Vec<(Box<Expression>, Box<Block>)>, Option<Box<Block>>),
}

impl<'a> DisplayWithOptions<IndentOptions<'a>> for Statement {
    fn fmt(&self, f: &mut Formatter, options: &IndentOptions) -> std::fmt::Result {
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
            Statement::Class(c) => write!(f, "{}", with_options(c.as_ref(), options)),
            Statement::Function(fun) => write!(f, "{}", with_options(fun.as_ref(), options)),
            Statement::IfThenElse(ifs, else_) => {
                for (idx, (condition, body)) in ifs.iter().enumerate() {
                    if idx == 0 {
                        write!(f, "if {}:\n", condition)?;
                    }
                    else {
                        write!(f, "elif {}:\n", condition)?;
                    }

                    let options = options.deeper();
                    let mut f = IndentingFormatter::new(f, &options.full_indentation);
                    let options = options.restart();

                    write!(f, "{}", with_options(body.as_ref(), &options))?;
                }
                if let Some(else_) = else_ {
                    write!(f, "else:\n")?;

                    let options = options.deeper();
                    let mut f = IndentingFormatter::new(f, &options.full_indentation);
                    let options = options.restart();

                    write!(f, "{}", with_options(else_.as_ref(), &options))?;
                }

                Ok(())
            }
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
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Expression::UnaryOperation(op, ex) => {
                if matches!(op.as_str(), "+" | "-") {
                    write!(f, "{}", op)?;
                }
                else {
                    write!(f, "{} ", op)?;
                }
                write_maybe_parenthesized(f, ex, !ex.is_simple())
            }
            Expression::BinaryOperation(lhs, op, rhs) => {
                write_maybe_parenthesized(f, lhs, !lhs.is_simple())?;
                write!(f, " {} ", op)?;
                write_maybe_parenthesized(f, rhs, !rhs.is_simple())
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
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.type_)
    }
}

pub fn write_maybe_parenthesized<D: Display>(f: &mut Formatter, d: D, parenthesize: bool) -> std::fmt::Result {
    if parenthesize {
        write!(f, "({})", d)
    }
    else {
        write!(f, "{}", d)
    }
}
