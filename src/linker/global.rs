use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use guard::guard;
use uuid::Uuid;
use crate::parser::abstract_syntax;
use crate::program::computation_tree::*;
use crate::linker::imperative::ImperativeLinker;
use crate::linker::{LinkError, scopes};
use crate::linker::conformance::ConformanceLinker;
use crate::linker::interface::{link_function_pointer, link_operator_pointer};
use crate::linker::scopes::Environment;
use crate::linker::traits::TraitLinker;
use crate::program::allocation::{ObjectReference, Reference, ReferenceType};
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

pub struct FunctionWithoutBody<'a> {
    pub pointer: Rc<FunctionPointer>,
    pub decorators: Vec<String>,
    pub body: &'a Vec<Box<abstract_syntax::Statement>>,
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

                self.add_function(FunctionWithoutBody {
                    pointer: fun,
                    decorators: syntax.decorators.clone(),
                    body,
                })?;
            }
            abstract_syntax::GlobalStatement::Operator(syntax) => {
                let scope = &self.global_variables;
                let fun = link_operator_pointer(&syntax, &scope, requirements)?;
                guard!(let Some(body) = &syntax.body else {
                    return Err(LinkError::LinkError { msg: format!("Function {} needs a body.", fun.name) });
                });

                self.add_function(FunctionWithoutBody {
                    pointer: Rc::clone(&fun),
                    decorators: syntax.decorators.clone(),
                    body,
                })?;
            }
            abstract_syntax::GlobalStatement::Trait(syntax) => {
                let mut trait_ = Trait::new(syntax.name.clone());

                let self_type = trait_.create_generic_type(&"self".into());
                let self_type_reference = Reference::make(ReferenceType::Object(ObjectReference::new_immutable(TypeProto::meta(self_type.clone()))));

                let mut scope = self.global_variables.subscope();
                scope.insert_singleton(Environment::Global, self_type_reference, &"Self".into());

                let mut linker = TraitLinker {
                    trait_: &mut trait_,
                    builtins: self.builtins,
                };
                for statement in syntax.statements.iter() {
                    linker.link_statement(statement, requirements, &scope)?;
                }

                let trait_ = Rc::new(trait_);
                let reference = self.module.add_trait(&trait_);

                self.global_variables.insert_singleton(
                    Environment::Global,
                    Reference::make(ReferenceType::Object(reference)),
                    &trait_.name.clone()
                );

            }
            abstract_syntax::GlobalStatement::Conformance(syntax) => {
                let target = self.global_variables.resolve(Environment::Global, &syntax.target).unwrap().as_trait().unwrap();
                let trait_ = self.global_variables.resolve(Environment::Global, &syntax.trait_).unwrap().as_trait().unwrap();

                let self_type = target.create_generic_type(&"self".into());
                let self_type_reference = Reference::make(ReferenceType::Object(ObjectReference::new_immutable(TypeProto::meta(self_type.clone()))));
                let self_binding = trait_.create_generic_binding(vec![(&"self".into(), self_type)]);

                let mut scope = self.global_variables.subscope();
                scope.insert_singleton(Environment::Global, self_type_reference, &"Self".into());

                let mut linker = ConformanceLinker {
                    binding: self_binding,
                    builtins: self.builtins,
                    functions: vec![],
                };
                for statement in syntax.statements.iter() {
                    linker.link_statement(statement, &requirements, &scope)?;
                }

                // TODO To be order independent, we should finalize after sorting...
                //  ... Or check inconsistencies only at the very end.
                linker.finalize(&mut self.module)?;
                for function in linker.functions {
                    self.add_function(function)?;
                }
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

    pub fn add_function(&mut self, fun: FunctionWithoutBody<'a>) -> Result<(), LinkError> {
        // Create a variable for the function
        self.module.add_function(&fun.pointer);
        self.global_variables.overload_function(&fun.pointer, &self.module.functions[&fun.pointer])?;

        self.functions.push(fun);

        // if interface.is_member_function {
        // TODO Create an additional variable as Metatype.function(self, ...args)?
        // }

        Ok(())
    }
}
