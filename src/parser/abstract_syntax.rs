use std::fmt::{Binary, Debug, Error, Formatter};
use itertools::zip_eq;
use crate::fmtutil::write_comma_separated_list;
use crate::program::types::{Mutability, ParameterKey};

// =============================== Global =====================================

pub struct Program {
    pub global_statements: Vec<Box<GlobalStatement>>
}

pub struct GlobalScope {
    pub generics: Option<Vec<String>>,
    pub requirements: Option<Vec<Box<TraitDeclaration>>>,
    pub statements: Vec<Box<GlobalStatement>>
}

pub enum GlobalStatement {
    FunctionDeclaration(Box<Function>),
    Operator(Box<Operator>),
    Pattern(Box<PatternDeclaration>),
    Scope(Box<GlobalScope>),
}

pub struct Function {
    pub target: Option<Box<ContextualParameter>>,
    pub identifier: String,
    pub parameters: Vec<Box<KeyedParameter>>,

    pub body: Vec<Box<Statement>>,
    pub return_type: Option<Box<TypeDeclaration>>,
}

pub struct KeyedParameter {
    pub key: ParameterKey,
    pub internal_name: String,
    pub param_type: Box<TypeDeclaration>,
}

pub struct Operator {
    pub lhs: Option<Box<ContextualParameter>>,
    pub operator: String,
    pub rhs: Box<ContextualParameter>,

    pub body: Vec<Box<Statement>>,
    pub return_type: Option<Box<TypeDeclaration>>,
}

pub struct ContextualParameter {
    pub internal_name: String,
    pub param_type: Box<TypeDeclaration>,
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

pub struct TraitDeclaration {
    pub unit: String,
    pub elements: Vec<Box<SpecializedType>>
}

pub struct SpecializedType {
    pub unit: String,
    pub elements: Option<Vec<Box<SpecializedType>>>
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PatternForm {
    Unary, Binary
}

// =============================== Type =====================================

pub enum TypeDeclaration {
    Identifier(String),
    Monad { unit: Box<TypeDeclaration>, shape: Vec<Box<Expression>> }
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
    Int(i128),
    Float(f64),
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
    pub key: ParameterKey,
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
        Expression::Int(_) => true,
        Expression::Float(_) => true,
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
        match *self {
            Mutable => "var",
            Immutable => "let",
        }
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
            Scope(scope) => {
                write!(fmt, "given <")?;
                for item in scope.generics.iter() { write!(fmt, "{:?},", item)? };
                write!(fmt, ">")?;
                if let Some(requirements) = &scope.requirements {
                    write!(fmt, "if ")?;
                    for item in requirements.iter() { write!(fmt, "{:?}, ", item)? };
                }
                write!(fmt, "{{")?;
                for item in scope.statements.iter() { write!(fmt, "{:?}\n\n", item)? };
                write!(fmt, "}}")?;
                return Ok(())
            }
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

impl Debug for TypeDeclaration {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::TypeDeclaration::*;
        match self {
            Identifier(name) => write!(fmt, "{}", name),
            Monad { unit, shape } => {
                write!(fmt, "{:?}[", unit)?;
                for item in shape { write!(fmt, "{:?},", item)? };
                write!(fmt, "]")?;
                return Ok(())
            }
        }
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

impl Debug for Expression {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Expression::*;
        match self {
            Int(n) => write!(fmt, "{:?}", n),
            Float(n) => write!(fmt, "{:?}", n),
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
                for (argument, operator) in zip_eq(arguments, operators) {
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

impl Debug for TraitDeclaration {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?}<", self.unit)?;
        write_comma_separated_list(fmt, &self.elements)?;
        write!(fmt, ">")?;
        return Ok(())
    }
}

impl Debug for SpecializedType {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?}", self.unit)?;
        if let Some(arguments) = &self.elements {
            write!(fmt, "<")?;
            write_comma_separated_list(fmt, arguments)?;
            write!(fmt, ">")?;
        }
        return Ok(())
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

impl TypeDeclaration {
    pub fn add_type_names<'a>(&'a self, items: &mut Vec<&'a String>) {
        match self {
            TypeDeclaration::Identifier(s) => items.push(s),
            TypeDeclaration::Monad { unit, shape } => {
                unit.add_type_names(items);
                // TODO Shape
            }
        }
    }
}

impl Function {
    pub fn gather_type_names<'a>(&'a self) -> Vec<&'a String> {
        let mut type_names = Vec::new();

        self.return_type.iter().for_each(|x| x.add_type_names(&mut type_names));
        self.parameters.iter().for_each(|x| x.param_type.add_type_names(&mut type_names));
        self.target.iter().for_each(|x| x.param_type.add_type_names(&mut type_names));

        type_names
    }
}

impl Operator {
    pub fn gather_type_names<'a>(&'a self) -> Vec<&'a String> {
        let mut type_names = Vec::new();

        self.return_type.iter().for_each(|x| x.add_type_names(&mut type_names));
        self.lhs.iter().for_each(|x| x.param_type.add_type_names(&mut type_names));
        self.rhs.param_type.add_type_names(&mut type_names);

        type_names
    }
}
