use itertools::Itertools;

use crate::ast;
use crate::error::{ErrInRange, RResult, RuntimeError, TryCollectMany};
use crate::interpreter::runtime::Runtime;
use crate::program::functions::ParameterKey;
use crate::program::module::ModuleName;
use crate::resolver::{interpreter_mock, scopes};
use crate::util::iter::omega;
use crate::util::position::Positioned;

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

pub fn resolve_imports(body: &Vec<Box<Positioned<ast::StructArgument>>>) -> RResult<Vec<Import>> {
    body.iter().map(|arg| {
        if arg.value.key != ParameterKey::Positional {
            return Err(
                RuntimeError::error("Imports cannot be renamed for now.").to_array()
            );
        }
        if arg.value.type_declaration.is_some() {
            return Err(
                RuntimeError::error("Imports cannot have type declarations.").to_array()
            );
        }

        resolve_module(&arg.value.value)
    }).try_collect_many()
}

pub fn resolve_module(body: &ast::Expression) -> RResult<Import> {
    let error = RuntimeError::error("Import parameter is not a module.").to_array();

    let [l, r] = &body[..] else {
        return Err(error);
    };

    let (ast::Term::MacroIdentifier(name), ast::Term::Struct(macro_struct)) = (&l.value, &r.value) else {
        return Err(error).err_in_range(&l.position);
    };

    if name != "module" {
        return Err(error).err_in_range(&l.position);
    }
    let body = interpreter_mock::plain_parameter("module!", macro_struct)?;
    let pterm = body.iter().exactly_one()
        .map_err(|_| error.clone()).err_in_range(&r.position)?;

    let ast::Term::StringLiteral(string) = &pterm.value else {
        return Err(error).err_in_range(&l.position);
    };

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

pub fn deep(runtime: &Runtime, module_name: ModuleName, scope: &mut scopes::Scope) -> RResult<()> {
    let all_modules = omega([&module_name].into_iter(), |m| runtime.source.module_by_name[*m].included_modules.iter());

    for module in all_modules {
        scope.import(&runtime.source.module_by_name[module], runtime)?;
    }

    Ok(())
}
