use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use guard::guard;
use itertools::{Itertools, zip_eq};
use uuid::Uuid;

use crate::parser;
use crate::parser::abstract_syntax;
use crate::program::computation_tree::*;
use crate::linker::imperative::ImperativeLinker;
use crate::linker::{LinkError, scopes};
use crate::linker::interface::{link_constant_pointer, link_function_pointer, link_operator_pointer};
use crate::linker::r#type::TypeFactory;
use crate::linker::scopes::Environment;
use crate::parser::abstract_syntax::{PatternDeclaration, Term};
use crate::program::traits::{Trait, TraitConformanceDeclaration, TraitConformanceRequirement, TraitConformanceScope};
use crate::program::{primitives, Program};
use crate::program::allocation::{Reference, ReferenceType};
use crate::program::builtins::*;
use crate::program::functions::{FunctionForm, FunctionPointer, FunctionPointerTarget, HumanFunctionInterface, MachineFunctionInterface, ParameterKey};
use crate::program::generics::TypeForest;
use crate::program::global::{FunctionImplementation, GlobalStatement};
use crate::program::types::*;
use crate::util::multimap::extend_multimap;


struct GlobalLinker<'a> {
    functions: Vec<FunctionWithoutBody<'a>>,
    traits: HashSet<Rc<Trait>>,
    global_variables: scopes::Scope<'a>,
    builtins: &'a Builtins,
}

struct FunctionWithoutBody<'a> {
    pointer: Rc<FunctionPointer>,
    body: &'a Vec<Box<abstract_syntax::Statement>>,
}

pub fn link_file(syntax: abstract_syntax::Program, scope: &scopes::Scope, builtins: &Builtins) -> Result<Program, LinkError> {
    let mut global_linker = GlobalLinker {
        functions: Vec::new(),
        traits: HashSet::new(),
        global_variables: scope.subscope(),
        builtins
    };

    // Resolve global types / interfaces
    for statement in &syntax.global_statements {
        global_linker.link_global_statement(statement.as_ref(), &HashSet::new())?;
    }

    let global_variable_scope = &global_linker.global_variables;
    let mut global_statements = vec![];
    let mut functions: HashSet<Rc<FunctionImplementation>> = HashSet::new();

    // Resolve function bodies
    for fun in global_linker.functions.iter() {
        let mut variable_names = HashMap::new();
        for (name, (_, ref_)) in zip_eq(fun.pointer.human_interface.parameter_names_internal.iter(), fun.pointer.human_interface.parameter_names.iter()) {
            variable_names.insert(Rc::clone(ref_), name.clone());
        }

        // TODO Inject traits, not pointers
        let mut resolver = Box::new(ImperativeLinker {
            function: Rc::clone(&fun.pointer),
            builtins,
            expressions: Box::new(ExpressionForest::new()),
            variable_names,
            ambiguities: vec![]
        });

        let implementation = resolver.link_function_body(fun.body, &global_variable_scope)?;
        functions.insert(Rc::clone(&implementation));
        global_statements.push(GlobalStatement::Function(implementation));
    }

    let main_function = functions.iter()
        .filter(|f| {
            f.human_interface.name == "main"
            && f.human_interface.form == FunctionForm::Global
            && f.human_interface.parameter_names.is_empty()
        })
        .map(Rc::clone)
        .next();

    Ok(Program {
        functions,
        traits: global_linker.traits.iter().map(Rc::clone).collect(),
        global_statements,
        main_function,
    })
}

impl <'a> GlobalLinker<'a> {
    pub fn link_global_statement(&mut self, statement: &'a abstract_syntax::GlobalStatement, requirements: &HashSet<Rc<TraitConformanceRequirement>>) -> Result<(), LinkError> {
        match statement {
            abstract_syntax::GlobalStatement::Pattern(pattern) => {
                let pattern = self.link_pattern(pattern)?;
                &self.global_variables.add_pattern(pattern);
            }
            abstract_syntax::GlobalStatement::FunctionDeclaration(syntax) => {
                let scope = &self.global_variables;
                let fun = link_function_pointer(&syntax, &scope, requirements)?;

                self.functions.push(FunctionWithoutBody {
                    pointer: Rc::clone(&fun),
                    body: &syntax.body,
                });

                // Create a variable for the function
                self.global_variables.overload_function(&fun);

                // if interface.is_member_function {
                // TODO Create an additional variable as Metatype.function(self, ...args)?
                // }
            }
            abstract_syntax::GlobalStatement::Operator(syntax) => {
                let scope = &self.global_variables;
                let fun = link_operator_pointer(&syntax, &scope, requirements)?;

                self.functions.push(FunctionWithoutBody {
                    pointer: Rc::clone(&fun),
                    body: &syntax.body,
                });

                // Create a variable for the function
                self.global_variables.overload_function(&fun);
            }
            abstract_syntax::GlobalStatement::Constant(syntax) => {
                let scope = &self.global_variables;
                let fun = link_constant_pointer(&syntax, &scope, requirements)?;

                self.functions.push(FunctionWithoutBody {
                    pointer: Rc::clone(&fun),
                    body: &syntax.body,
                });

                // Create a variable for the function
                self.global_variables.insert_constant(fun);
            }
        }

        Ok(())
    }

    pub fn link_pattern(&mut self, syntax: &PatternDeclaration) -> Result<Rc<Pattern>, LinkError> {
        let precedence_group = self.global_variables.resolve_precedence_group(&syntax.precedence);

        Ok(Rc::new(Pattern {
            id: Uuid::new_v4(),
            alias: syntax.alias.clone(),
            precedence_group,
            parts: syntax.parts.clone(),
        }))
    }
}