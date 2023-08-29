use std::collections::HashSet;
use std::rc::Rc;
use crate::linker::{LinkError, scopes};
use crate::linker::interface::link_function_pointer;
use crate::parser::abstract_syntax;
use crate::program::builtins::Builtins;
use crate::program::module::Module;
use crate::program::traits::{Trait, TraitBinding};

pub struct TraitLinker<'a> {
    pub trait_: &'a mut Trait,

    pub builtins: &'a Builtins,
}

impl <'a> TraitLinker<'a> {
    pub fn link_statement(&mut self, statement: &'a abstract_syntax::GlobalStatement, requirements: &HashSet<Rc<TraitBinding>>, scope: &scopes::Scope) -> Result<(), LinkError> {
        match statement {
            abstract_syntax::GlobalStatement::FunctionDeclaration(syntax) => {
                let fun = link_function_pointer(&syntax, &scope, requirements)?;
                if !syntax.body.is_none() {
                    return Err(LinkError::LinkError { msg: format!("Abstract function {} cannot have a body.", fun.name) });
                };

                self.trait_.abstract_functions.insert(fun);
            }
            _ => {
                return Err(LinkError::LinkError { msg: format!("Statement {:?} not valid in a trait context.", statement) });
            }
        }

        Ok(())
    }
}
