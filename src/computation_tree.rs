use std::borrow::Borrow;
use crate::abstract_syntax;

pub struct Program {
    pub functions: Vec<Box<Function>>
}

pub struct Function {
    pub identifier: String,
    pub return_type: Option<Box<Type>>
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

pub fn analyze_type(syntax: &abstract_syntax::TypeDeclaration) -> Box<Type> {
    Box::new(match syntax.borrow() {
        abstract_syntax::TypeDeclaration::Identifier(id) => Type::Identifier(id.clone()),
        abstract_syntax::TypeDeclaration::NDArray(identifier, _) => {
            Type::NDArray(analyze_type(&identifier))
        }
    })
}
