use std::fmt::{Debug, Error, Formatter};

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

#[derive(Clone)]
pub enum ParameterKey {
    Name(String),
    Int(i32),
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
    Bool(bool),
    BinaryOperator { lhs: Box<Expression>, operator: BinaryOperator, rhs: Box<Expression> },
    UnaryOperator(UnaryOperator, Box<Expression>),
    FunctionCall(FunctionCallType, Box<Expression>, Vec<Box<PassedArgument>>),
    MemberLookup(Box<Expression>, String),
    VariableLookup(String),
    ArrayLiteral(Vec<Box<Expression>>),
    StringLiteral(String),
}

pub struct PassedArgument {
    pub key: Option<ParameterKey>,
    pub value: Box<Expression>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,
    Positive,
    Negative,
}

#[derive(Copy, Clone, PartialEq)]
pub enum BinaryOperator {
    Or,
    And,

    EqualTo,
    NotEqualTo,

    GreaterThan,
    GreaterThanOrEqualTo,
    LesserThan,
    LesserThanOrEqualTo,

    Multiply,
    Divide,
    Add,
    Subtract,
    Exponentiate,
    Modulo,
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
            BinaryOperator { lhs, operator, rhs } => write!(fmt, "({:?}) {:?} ({:?})", lhs, operator, rhs),
            UnaryOperator(op, expression) => write!(fmt, "{:?}{:?}", op, expression),
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
            Bool(value) => write!(fmt, "{}", value),
        }
    }
}

impl Debug for BinaryOperator {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::BinaryOperator::*;
        match self {
            Or => write!(fmt, "||"),
            And => write!(fmt, "&&"),

            EqualTo => write!(fmt, "=="),
            NotEqualTo => write!(fmt, "!="),

            GreaterThan => write!(fmt, ">"),
            GreaterThanOrEqualTo => write!(fmt, ">="),
            LesserThan => write!(fmt, "<"),
            LesserThanOrEqualTo => write!(fmt, "<="),

            Multiply => write!(fmt, "*"),
            Divide => write!(fmt, "/"),
            Add => write!(fmt, "+"),
            Subtract => write!(fmt, "-"),
            Exponentiate => write!(fmt, "**"),
            Modulo => write!(fmt, "%"),
        }
    }
}

impl Debug for UnaryOperator {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        use self::UnaryOperator::*;
        match self {
            Not => write!(fmt, "!"),
            Positive => write!(fmt, "+"),
            Negative => write!(fmt, "-"),
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

impl Debug for ParameterKey {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::ParameterKey::*;
        match self {
            Int(value) => write!(fmt, "{}", value),
            Name(value) => write!(fmt, "{}", value),
        }
    }
}

impl BinaryOperator {
    pub fn is_pairwise_comparison(&self) -> bool {
        match self {
            BinaryOperator::EqualTo => true,
            BinaryOperator::NotEqualTo => true,
            BinaryOperator::GreaterThan => true,
            BinaryOperator::GreaterThanOrEqualTo => true,
            BinaryOperator::LesserThan => true,
            BinaryOperator::LesserThanOrEqualTo => true,
            _ => false
        }
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
