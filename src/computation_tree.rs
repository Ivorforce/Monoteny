use std::borrow::Borrow;
use crate::abstract_syntax;

pub struct Program {
    pub functions: Vec<Box<Function>>
}

pub struct Function {
    pub identifier: String
}

pub fn analyze_program(syntax: abstract_syntax::Program) -> Program {
    let mut functions: Vec<Box<Function>> = Vec::new();

    for statement in syntax.global_statements {
        match statement.borrow() {
            abstract_syntax::GlobalStatement::FunctionDeclaration(function) => {
                functions.push(Box::new(Function {
                    identifier: function.identifier.clone()
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
