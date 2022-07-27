use std::borrow::Borrow;
use crate::abstract_syntax;

pub struct Program {
    pub functions: Vec<Box<Function>>
}

pub struct Function {
    pub identifier: String,
    pub parameters: Vec<Box<Parameter>>,
    pub return_type: Option<Box<Type>>
}

pub struct Parameter {
    pub external_name: String,
    pub variable: Box<Variable>
}

pub struct Variable {
    pub name: String,
    pub type_declaration: Box<Type>
}

pub enum Type {
    Identifier(String),
    NDArray(Box<Type>),
}

pub fn analyze_program(syntax: abstract_syntax::Program) -> Program {
    let mut functions: Vec<Box<Function>> = Vec::new();

    for statement in syntax.global_statements {
        match *statement {
            abstract_syntax::GlobalStatement::FunctionDeclaration(function) => {
                functions.push(Box::new(Function {
                    identifier: function.identifier.clone(),
                    parameters: function.parameters.into_iter().map(|x| analyze_parameter(&x)).collect(),
                    return_type: function.return_type.map(|x| analyze_type(&x))
                }));
            }
            abstract_syntax::GlobalStatement::Extension(extension) => {
                // TODO
            }
        }
    }

    return Program {
        functions
    }
}

pub fn analyze_parameter(syntax: &abstract_syntax::Parameter) -> Box<Parameter> {
    Box::new(Parameter {
        external_name: syntax.external_name.clone(),
        variable: Box::new(Variable {
            name: syntax.internal_name.clone(),
            type_declaration: analyze_type(syntax.param_type.as_ref())
        })
    })
}

pub fn analyze_type(syntax: &abstract_syntax::TypeDeclaration) -> Box<Type> {
    Box::new(match syntax.borrow() {
        abstract_syntax::TypeDeclaration::Identifier(id) => Type::Identifier(id.clone()),
        abstract_syntax::TypeDeclaration::NDArray(identifier, _) => {
            Type::NDArray(analyze_type(&identifier))
        }
    })
}
