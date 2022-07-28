use std::fmt::{Debug, Error, Formatter};

// =============================== Global =====================================

pub struct Program {
    pub global_statements: Vec<Box<GlobalStatement>>
}

pub enum GlobalStatement {
    FunctionDeclaration(Box<Function>),
    Extension(Box<Extension>),
}

pub struct Function {
    pub identifier: String,
    pub parameters: Vec<Box<Parameter>>,
    pub return_type: Option<Box<TypeDeclaration>>,
    pub body: Vec<Box<Statement>>
}

pub struct Parameter {
    pub internal_name: String,
    pub external_name: String,
    pub param_type: Box<TypeDeclaration>,
}

pub struct Extension {
    pub type_declaration: Box<TypeDeclaration>,
    pub statements: Vec<Box<MemberStatement>>
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
    VariableAssignment(String, Box<Expression>),
    Expression(Box<Expression>),
    Return(Option<Box<Expression>>),
}

pub enum Expression {
    Number(i32),
    BinaryOperator(Box<Expression>, Opcode, Box<Expression>),
    FunctionCall(FunctionCallType, Box<Expression>, Vec<Box<PassedArgument>>),
    MemberLookup(Box<Expression>, String),
    VariableLookup(String),
    ArrayLiteral(Vec<Box<Expression>>),
    StringLiteral(String),
    Error,
}

pub struct PassedArgument {
    pub name: Option<String>,
    pub value: Box<Expression>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Opcode {
    Multiply,
    Divide,
    Add,
    Subtract,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Mutability {
    Immutable,
    Mutable,
}

#[derive(Copy, Clone, PartialEq)]
pub enum FunctionCallType {
    Call,
    Subscript,
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

impl Debug for GlobalStatement {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::GlobalStatement::*;
        match self {
            FunctionDeclaration(function) => write!(fmt, "{:?}", function),
            Extension(extension) => write!(fmt, "{:?}", extension),
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

impl Debug for Extension {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "extension {:?} {{", self.type_declaration)?;
        for item in self.statements.iter() {
            write!(fmt, "\n{:?}\n", item)?
        };
        write!(fmt, "}}")?;
        return Ok(())
    }
}

impl Debug for Function {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "fn {}(", self.identifier)?;
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
            VariableAssignment(id, expr) => {
                write!(fmt, "{} = {:?}", id, expr)
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
            FunctionCall(call_type, expression, args) => {
                let brackets = call_type.bracket_str();
                write!(fmt, "{:?}{}", expression, brackets.chars().nth(0).unwrap())?;
                for item in args { write!(fmt, "{:?},", item)? };
                write!(fmt, "{}", brackets.chars().nth(1).unwrap())?;
                return Ok(())
            },
            VariableLookup(id) => write!(fmt, "{}", id),
            MemberLookup(expression, id) => write!(fmt, "{:?}.{}", expression, id),
            ArrayLiteral(items) => {
                write!(fmt, "[")?;
                for item in items { write!(fmt, "{:?},", item)? };
                write!(fmt, "]")?;
                return Ok(())
            },
            StringLiteral(string) => write!(fmt, "{:?}", string),
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

impl Debug for PassedArgument {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match &self.name {
            Some(name) => write!(fmt, "{}: {:?}", name, self.value),
            None => write!(fmt, "{:?}", self.value)
        }
    }
}

impl Debug for Parameter {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{} {}: {:?}", self.external_name, self.internal_name, self.param_type)
    }
}

impl Mutability {
    fn variable_declaration_keyword(&self) -> &str {
        use self::Mutability::*;
        return match *self {
            Mutable => "var",
            Immutable => "let",
        };
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
