use std::borrow::Borrow;
use std::collections::HashMap;
use std::env::var;
use std::rc::Rc;
use uuid::Uuid;
use crate::abstract_syntax;

// ================================ Global ==============================

pub struct Program {
    pub functions: Vec<Box<Function>>,
    pub variables: HashMap<Uuid, Rc<Variable>>,
}

pub struct Function {
    pub identifier: String,
    pub parameters: Vec<Box<Parameter>>,
    pub return_type: Option<Box<Type>>,

    pub variables: HashMap<Uuid, Rc<Variable>>,
    pub statements: Vec<Box<Statement>>,
}

pub struct Parameter {
    pub external_name: String,
    pub variable: Rc<Variable>
}

// ================================ Type ==============================

#[derive(Clone)]
pub struct Variable {
    pub id: Uuid,
    pub home: VariableHome,
    pub name: String,
    pub type_declaration: Box<Type>,
}

#[derive(Clone)]
pub enum Type {
    Identifier(String),
    NDArray(Box<Type>),
}

// ================================ Code ==============================

#[derive(Copy, Clone)]
pub enum VariableHome {
    Local, Global
}

pub enum Statement {
    VariableAssignment(String, Box<Expression>),
    Expression(Box<Expression>),
    Return(Box<Expression>),
}

pub enum Expression {
    Number(i32),
    BinaryOperator(Box<Expression>, Opcode, Box<Expression>),
    FunctionCall(FunctionCallType, Box<Expression>, Vec<Box<PassedArgument>>),
    MemberLookup(Box<Expression>, String),
    VariableLookup(Rc<Variable>),
    ArrayLiteral(Vec<Box<Expression>>),
    StringLiteral(String),
}

pub struct PassedArgument {
    pub name: Option<String>,
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
pub enum FunctionCallType {
    Call,
    Subscript,
}

pub fn analyze_program(syntax: abstract_syntax::Program) -> Program {
    let mut functions: Vec<Box<Function>> = Vec::new();

    for statement in syntax.global_statements {
        match *statement {
            abstract_syntax::GlobalStatement::FunctionDeclaration(function) => {
                functions.push(analyze_function(&function));
            }
            abstract_syntax::GlobalStatement::Extension(extension) => {
                // TODO
            }
        }
    }

    return Program {
        variables: HashMap::new(),
        functions
    }
}

pub fn analyze_function(function: &abstract_syntax::Function) -> Box<Function> {
    let mut variables: HashMap<Uuid, Rc<Variable>> = HashMap::new();
    let mut parameters: Vec<Box<Parameter>> = Vec::new();

    for parameter in function.parameters.iter() {
        let variable = Rc::new(Variable {
            id: Uuid::new_v4(),
            home: VariableHome::Local,
            name: parameter.internal_name.clone(),
            type_declaration: analyze_type(parameter.param_type.as_ref()),
        });

        variables.insert(variable.id, variable.clone());
        parameters.push(Box::new(Parameter {
            external_name: parameter.external_name.clone(),
            variable
        }));
    }

    return Box::new(Function {
        identifier: function.identifier.clone(),
        parameters,
        variables,
        statements: Vec::new(),
        return_type: function.return_type.as_ref().map(|x| analyze_type(&x))
    });
}

pub fn analyze_type(syntax: &abstract_syntax::TypeDeclaration) -> Box<Type> {
    Box::new(match syntax.borrow() {
        abstract_syntax::TypeDeclaration::Identifier(id) => Type::Identifier(id.clone()),
        abstract_syntax::TypeDeclaration::NDArray(identifier, _) => {
            Type::NDArray(analyze_type(&identifier))
        }
    })
}
