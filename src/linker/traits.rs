use std::collections::HashSet;
use std::rc::Rc;
use crate::interpreter::InterpreterError;
use crate::linker::scopes;
use crate::linker::interface::link_function_pointer;
use crate::parser::ast;
use crate::program::traits::{Trait, TraitBinding};

pub struct TraitLinker<'a> {
    pub trait_: &'a mut Trait,
}

impl <'a> TraitLinker<'a> {
    pub fn link_statement(&mut self, statement: &'a ast::GlobalStatement, requirements: &HashSet<Rc<TraitBinding>>, scope: &scopes::Scope) -> Result<(), InterpreterError> {
        match statement {
            ast::GlobalStatement::FunctionDeclaration(syntax) => {
                let fun = link_function_pointer(&syntax, &scope, requirements)?;
                if !syntax.body.is_none() {
                    return Err(InterpreterError::LinkerError { msg: format!("Abstract function {} cannot have a body.", fun.name) });
                };

                self.trait_.insert_function(fun);
            }
            _ => {
                return Err(InterpreterError::LinkerError { msg: format!("Statement {} not valid in a trait context.", statement) });
            }
        }

        Ok(())
    }
}
