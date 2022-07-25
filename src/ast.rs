use std::fmt::{Debug, Error, Formatter};

pub enum GlobalStatement {
    FunctionDeclaration(String, Vec<Box<Statement>>),
}

pub enum Statement {
    VariableDeclaration(Mutability, String, Box<Expression>),
}

pub enum Expression {
    Number(i32),
    BinaryOperator(Box<Expression>, Opcode, Box<Expression>),
    Error,
}

#[derive(Copy, Clone)]
pub enum Opcode {
    Multiply,
    Divide,
    Add,
    Subtract,
}

#[derive(Copy, Clone)]
pub enum Mutability {
    Immutable,
    Mutable,
}

impl Debug for Statement {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Statement::*;
        match self {
            VariableDeclaration(Mutability::Mutable, id, expr) => write!(fmt, "var {} = {:?}", id, expr),
            VariableDeclaration(Mutability::Immutable, id, expr) => write!(fmt, "let {} = {:?}", id, expr),
        }
    }
}

impl Debug for Expression {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Expression::*;
        match *self {
            Number(n) => write!(fmt, "{:?}", n),
            BinaryOperator(ref l, op, ref r) => write!(fmt, "({:?} {:?} {:?})", l, op, r),
            Error => write!(fmt, "error"),
        }
    }
}

impl Debug for Opcode {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Opcode::*;
        match *self {
            Multiply => write!(fmt, "*"),
            Divide => write!(fmt, "/"),
            Add => write!(fmt, "+"),
            Subtract => write!(fmt, "-"),
        }
    }
}

impl Debug for GlobalStatement {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::GlobalStatement::*;
        match self {
            FunctionDeclaration(id, stmts) => write!(fmt, "fn {}() {{{:?}}}", id, stmts),
        }
    }
}
