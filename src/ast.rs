use std::fmt::{Debug, Error, Formatter};

// =============================== Global =====================================

pub struct Program {
    pub global_statements: Vec<Box<GlobalStatement>>
}

pub enum GlobalStatement {
    FunctionDeclaration(String, Vec<Box<ParameterDeclaration>>, Vec<Box<Statement>>),
}

pub struct ParameterDeclaration {
    pub internal_name: String,
    pub external_name: String,
    pub param_type: Box<TypeDeclaration>,
}

// =============================== Type =====================================

pub enum TypeDeclaration {
    Identifier(String),
    NDArray(String, Vec<Box<Expression>>)
}

// =============================== Body =====================================

pub enum Statement {
    VariableDeclaration(Mutability, String, Option<Box<TypeDeclaration>>, Box<Expression>),
    Expression(Box<Expression>),
    Return(Box<Expression>),
}

pub enum Expression {
    Number(i32),
    BinaryOperator(Box<Expression>, Opcode, Box<Expression>),
    FunctionCall(Box<Expression>, Vec<Box<PassedArgument>>),
    MemberLookup(Box<Expression>, String),
    VariableLookup(String),
    ArrayLiteral(Vec<Box<Expression>>),
    Error,
}

pub struct PassedArgument {
    pub name: String,
    pub value: Box<Expression>,
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

// =============================== String =====================================

impl Debug for Program {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        for item in self.global_statements.iter() {
            write!(fmt, "{:?}\n\n", item)?
        };
        return Ok(())
    }
}

impl Debug for Statement {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Statement::*;
        match self {
            VariableDeclaration(mutability, id, type_name, expr) => {
                let mutabilityString = mutability.variableDeclarationKeyword();
                write!(fmt, "{} {}: {:?} = {:?}", mutabilityString, id, type_name, expr)
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
            BinaryOperator(ref l, op, ref r) => write!(fmt, "({:?} {:?} {:?})", l, op, r),
            FunctionCall(expression, args) => {
                write!(fmt, "{:?}(", expression);
                for item in args { write!(fmt, "{:?},", item)? };
                write!(fmt, ")");
                return Ok(())
            },
            VariableLookup(id) => write!(fmt, "{}", id),
            MemberLookup(expression, id) => write!(fmt, "{:?}.{}", expression, id),
            ArrayLiteral(items) => {
                write!(fmt, "[");
                for item in items { write!(fmt, "{:?},", item)? };
                write!(fmt, "]");
                return Ok(())
            },
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
            FunctionDeclaration(id, params, stmts) => {
                write!(fmt, "fn {}(", id);
                for item in params { write!(fmt, "{:?},", item)? };
                write!(fmt, ") {{\n");
                for item in stmts { write!(fmt, "    {:?};\n", item)? };
                write!(fmt, "}}");
                return Ok(())
            },
        }
    }
}

impl Debug for TypeDeclaration {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::TypeDeclaration::*;
        match self {
            Identifier(name) => write!(fmt, "{}", name),
            NDArray(name, dimensions) => {
                write!(fmt, "{}[", name);
                for item in dimensions { write!(fmt, "{:?},", item)? };
                write!(fmt, "]");
                return Ok(())
            },
        }
    }
}

impl Debug for PassedArgument {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{}: {:?}", self.name, self.value)
    }
}

impl Debug for ParameterDeclaration {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{} {}: {:?}", self.external_name, self.internal_name, self.param_type)
    }
}

impl Mutability {
    fn variableDeclarationKeyword(&self) -> &str {
        use self::Mutability::*;
        return match *self {
            Mutable => "var",
            Immutable => "let",
        };
    }
}

