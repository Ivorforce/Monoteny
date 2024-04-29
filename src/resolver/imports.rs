use itertools::Itertools;

use crate::ast;
use crate::error::{ErrInRange, RResult, RuntimeError, TryCollectMany};
use crate::interpreter::runtime::Runtime;
use crate::parser::expressions;
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

pub fn resolve_imports(body: &ast::Struct, scope: &scopes::Scope) -> RResult<Vec<Import>> {
    body.arguments.iter().map(|arg| {
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

        resolve_module(&arg.value.value, scope)
    }).try_collect_many()
}

pub fn resolve_module(body: &ast::Expression, scope: &scopes::Scope) -> RResult<Import> {
    let error = RuntimeError::error("Import parameter is not a module.").to_array();

    let parsed = expressions::parse(body, &scope.grammar)?;

    let expressions::Value::FunctionCall(target, call_struct) = &parsed.value else {
        return Err(error)
    };

    let expressions::Value::MacroIdentifier(name) = &target.value else {
        return Err(error)
    };

    if name.as_str() != "module" {
        return Err(error).err_in_range(&target.position);
    }
    let body = interpreter_mock::plain_parameter("module!", call_struct)?;

    let argument_parsed = expressions::parse(body, &scope.grammar)?;

    let expressions::Value::StringLiteral(parts) = &argument_parsed.value else {
        return Err(error);
    };

    let mut literal = interpreter_mock::plain_string_literal("module!", parts)?;

    let is_relative = literal.starts_with(".");
    if is_relative {
        let mut chars = literal.chars();
        chars.next();
        literal = chars.as_str();
    }

    let mut elements = literal.split(".").collect_vec();

    if !elements.iter().all(|p| p.chars().all(|c| c.is_alphanumeric())) {
        return Err(error);
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
