use itertools::Itertools;

use crate::error::{ErrInRange, RResult, RuntimeError};
use crate::interpreter::Runtime;
use crate::resolver::{interpreter_mock, scopes};
use crate::parser::ast;
use crate::program::functions::ParameterKey;
use crate::program::module::ModuleName;
use crate::util::iter::omega;

pub struct Import {
    pub is_relative: bool,
    pub elements: Vec<String>,
}

impl Import {
    pub fn relative_to(&self, path: &Vec<String>) -> Vec<String> {
        match self.is_relative {
            true => path.iter().chain(&self.elements).cloned().collect_vec(),
            false => self.elements.clone(),
        }
    }
}

pub fn resolve_imports(body: &Vec<ast::StructArgument>) -> RResult<Vec<Import>> {
    body.iter().map(|arg| {
        if arg.key != ParameterKey::Positional {
            return Err(RuntimeError::new(format!("Imports cannot be renamed for now.")));
        }
        if arg.type_declaration.is_some() {
            return Err(RuntimeError::new(format!("Imports cannot have type declarations.")));
        }

        resolve_module(&arg.value)
    }).try_collect()
}

pub fn resolve_module(body: &ast::Expression) -> RResult<Import> {
    let error = RuntimeError::new(format!("Import parameter is not a module."));

    match &body[..] {
        [l, r] => {
            match (&l.value, &r.value) {
                (ast::Term::MacroIdentifier(name), ast::Term::Struct(struct_args)) => {
                    if name != "module" {
                        return Err(error).err_in_range(&l.position);
                    }
                    let body = interpreter_mock::plain_parameter("module!", struct_args)?;
                    let pterm = body.iter().exactly_one()
                        .map_err(|_| error.clone()).err_in_range(&r.position)?;

                    match &pterm.value {
                        ast::Term::StringLiteral(string) => {
                            let literal = interpreter_mock::plain_string_literal("module!", string).err_in_range(&pterm.position)?;

                            if literal == "." {
                                return Ok(Import {
                                    is_relative: true,
                                    elements: vec![],
                                })
                            }

                            let mut elements = literal.split(".").collect_vec();
                            let is_relative = match elements.first() {
                                Some(p) => {
                                    if p == &"" {
                                        elements.remove(0);
                                        true
                                    }
                                    else {
                                        false
                                    }
                                }
                                None => return Err(error).err_in_range(&l.position),
                            };

                            if !elements.iter().skip(1).all(|p| p.chars().all(|c| c.is_alphanumeric())) {
                                return Err(error).err_in_range(&l.position);
                            }

                            Ok(Import {
                                is_relative,
                                elements: elements.iter().map(|e| e.to_string()).collect_vec(),
                            })
                        }
                        _ => Err(error).err_in_range(&l.position),
                    }
                }
                _ => Err(error).err_in_range(&l.position)
            }
        }
        _ => Err(error),
    }
}

pub fn deep(runtime: &Runtime, module_name: ModuleName, scope: &mut scopes::Scope) -> RResult<()> {
    let all_modules = omega([&module_name].into_iter(), |m| runtime.source.module_by_name[*m].included_modules.iter());

    for module in all_modules {
        scope.import(&runtime.source.module_by_name[module], runtime)?;
    }

    Ok(())
}
