use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use guard::guard;
use uuid::Uuid;
use crate::parser::abstract_syntax;
use crate::program::computation_tree::*;
use crate::linker::imperative::ImperativeLinker;
use crate::linker::{LinkError, scopes};
use crate::linker::interface::{link_function_pointer, link_operator_pointer};
use crate::linker::scopes::Environment;
use crate::program::allocation::{Reference, ReferenceType};
use crate::program::traits::{Trait, TraitBinding};
use crate::program::builtins::*;
use crate::program::functions::FunctionPointer;
use crate::program::generics::TypeForest;
use crate::program::module::Module;
use crate::program::types::*;

struct GlobalLinker<'a> {
    functions: Vec<FunctionWithoutBody<'a>>,
    module: Module,
    global_variables: scopes::Scope<'a>,
    builtins: &'a Builtins,
}

struct FunctionWithoutBody<'a> {
    pointer: Rc<FunctionPointer>,
    decorators: Vec<String>,
    body: &'a Vec<Box<abstract_syntax::Statement>>,
}

pub fn link_file(syntax: abstract_syntax::Program, scope: &scopes::Scope, builtins: &Builtins) -> Result<Rc<Module>, LinkError> {
    let mut global_linker = GlobalLinker {
        functions: Vec::new(),
        module: Module::new("main".into()),  // TODO Give it a name!
        global_variables: scope.subscope(),
        builtins
    };

    // Resolve global types / interfaces
    for statement in &syntax.global_statements {
        global_linker.link_global_statement(statement.as_ref(), &HashSet::new())?;
    }

    let global_variable_scope = &global_linker.global_variables;

    // Resolve function bodies
    for fun in global_linker.functions.iter() {
        let mut variable_names = HashMap::new();

        // TODO Inject traits, not pointers
        let mut resolver = Box::new(ImperativeLinker {
            function: Rc::clone(&fun.pointer),
            decorators: fun.decorators.clone(),
            builtins,
            types: Box::new(TypeForest::new()),
            expressions: Box::new(ExpressionForest::new()),
            variable_names,
            ambiguities: vec![]
        });

        let implementation = resolver.link_function_body(fun.body, &global_variable_scope)?;

        global_linker.module.function_implementations.insert(Rc::clone(&fun.pointer), Rc::clone(&implementation));
    }

    Ok(Rc::new(global_linker.module))
}

impl <'a> GlobalLinker<'a> {
    pub fn link_global_statement(&mut self, statement: &'a abstract_syntax::GlobalStatement, requirements: &HashSet<Rc<TraitBinding>>) -> Result<(), LinkError> {
        match statement {
            abstract_syntax::GlobalStatement::Pattern(pattern) => {
                let pattern = self.link_pattern(pattern)?;
                &self.global_variables.add_pattern(pattern);
            }
            abstract_syntax::GlobalStatement::FunctionDeclaration(syntax) => {
                let scope = &self.global_variables;
                let fun = link_function_pointer(&syntax, &scope, requirements)?;
                guard!(let Some(body) = &syntax.body else {
                    return Err(LinkError::LinkError { msg: format!("Function {} needs a body.", fun.name) });
                });

                self.functions.push(FunctionWithoutBody {
                    pointer: Rc::clone(&fun),
                    decorators: syntax.decorators.clone(),
                    body,
                });

                // Create a variable for the function
                self.module.add_function(&fun);
                self.global_variables.overload_function(&fun, &self.module.functions[&fun])?;

                // if interface.is_member_function {
                // TODO Create an additional variable as Metatype.function(self, ...args)?
                // }
            }
            abstract_syntax::GlobalStatement::Operator(syntax) => {
                let scope = &self.global_variables;
                let fun = link_operator_pointer(&syntax, &scope, requirements)?;
                guard!(let Some(body) = &syntax.body else {
                    return Err(LinkError::LinkError { msg: format!("Function {} needs a body.", fun.name) });
                });

                self.functions.push(FunctionWithoutBody {
                    pointer: Rc::clone(&fun),
                    decorators: syntax.decorators.clone(),
                    body,
                });

                // Create a variable for the function
                self.module.add_function(&fun);
                self.global_variables.overload_function(&fun, &self.module.functions[&fun])?;
            }
            abstract_syntax::GlobalStatement::Trait(syntax) => {
                let mut trait_ = Trait::new(syntax.name.clone());

                for statement in syntax.statements.iter() {
                    self.link_trait_statement(statement, &mut trait_, requirements)?;
                }

                let trait_ = Rc::new(trait_);
                let reference = self.module.add_trait(&trait_);

                self.global_variables.insert_singleton(
                    Environment::Global,
                    Reference::make(ReferenceType::Object(reference)),
                    &trait_.name.clone()
                );

            }
        }

        Ok(())
    }

    pub fn link_trait_statement(&mut self, statement: &'a abstract_syntax::TraitStatement, trait_: &mut Trait, requirements: &HashSet<Rc<TraitBinding>>) -> Result<(), LinkError> {
        match statement {
            abstract_syntax::TraitStatement::FunctionDeclaration(syntax) => {
                let scope = &self.global_variables;
                let fun = link_function_pointer(&syntax, &scope, requirements)?;
                if !syntax.body.is_none() {
                    return Err(LinkError::LinkError { msg: format!("Abstract function {} cannot have a body.", fun.name) });
                };

                trait_.abstract_functions.insert(fun);
            }
        }

        Ok(())
    }

    pub fn link_pattern(&mut self, syntax: &abstract_syntax::PatternDeclaration) -> Result<Rc<Pattern>, LinkError> {
        let precedence_group = self.global_variables.resolve_precedence_group(&syntax.precedence);

        Ok(Rc::new(Pattern {
            id: Uuid::new_v4(),
            alias: syntax.alias.clone(),
            precedence_group,
            parts: syntax.parts.clone(),
        }))
    }
}
