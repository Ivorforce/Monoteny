use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use crate::error::{RResult, RuntimeError};
use crate::interpreter::Runtime;
use crate::linker::scopes;
use crate::linker::scopes::Environment;
use crate::parser::ast;
use crate::program::traits::{Trait, TraitBinding};
use crate::program::types::{TypeProto, TypeUnit};


pub struct TypeFactory<'a> {
    pub scope: &'a scopes::Scope<'a>,
    pub generics: HashMap<String, Rc<Trait>>,
    pub requirements: HashSet<Rc<TraitBinding>>,
    pub runtime: &'a Runtime,
}

// TODO Essentially this is a form of mini interpreter.
//  In the future it might be easier to rewrite it as such.
impl <'a> TypeFactory<'a> {
    pub fn new(scope: &'a scopes::Scope<'a>, runtime: &'a Runtime) -> TypeFactory<'a> {
        TypeFactory {
            scope,
            generics: HashMap::new(),
            requirements: HashSet::new(),
            runtime,
        }
    }

    pub fn resolve_trait(&mut self, name: &str) -> RResult<Rc<Trait>> {
        let reference = self.scope.resolve(Environment::Global, &name)?;
        let overload = reference.as_function_overload()?;

        let function = overload.iter_heads().exactly_one()
            .map_err(|_| RuntimeError::new("Function overload cannot be resolved to a type.".to_string()))?;
        let trait_ = self.runtime.source.trait_references.get(function)
            .ok_or_else(|| RuntimeError::new("Interpreted types aren't supported yet; please use an explicit type for now. 1".to_string()))?;

        return Ok(Rc::clone(trait_))
    }

    fn register_generic(&mut self, name: &str) -> Rc<Trait> {
        let trait_ = Rc::new(Trait::new_flat(name.to_string()));
        self.generics.insert(name.to_string(), Rc::clone(&trait_));
        trait_
    }

    fn register_requirement(&mut self, requirement: Rc<TraitBinding>) {
        self.requirements.insert(requirement);
    }

    pub fn link_type(&mut self, syntax: &ast::Expression, allow_anonymous_generics: bool) -> RResult<Box<TypeProto>> {
        guard!(let Ok(pterm) = syntax.iter().exactly_one() else {
            return Err(RuntimeError::new("Interpreted types aren't supported yet; please use an explicit type for now. 2 ".to_string()));
        });
        let term: &ast::Term = &pterm.value;
        let arguments = vec![];

        match term {
            ast::Term::Identifier(type_name) => {
                if let Some(type_) = self.generics.get(type_name) {
                    return Ok(TypeProto::unit(TypeUnit::Struct(Rc::clone(type_))))
                }

                if !allow_anonymous_generics || !(type_name.starts_with("#") || type_name.starts_with("$")) {
                    // No special generic; let's try just resolving it normally.
                    let trait_ = self.resolve_trait(type_name)?;
                    // Found a trait! Until we actually interpret the expression, this is guaranteed to be unbound.
                    return Ok(TypeProto::unit(TypeUnit::Struct(trait_)));
                }

                let type_ = Box::new(TypeProto {
                    unit: TypeUnit::Struct(self.register_generic(type_name).clone()),
                    arguments
                });

                if type_name.starts_with("$") {
                    let type_name = match type_name.find("#") {
                        None => { String::from(&type_name[1..]) }
                        Some(hash_start_index) => { String::from(&type_name[1..hash_start_index]) }
                    };

                    let requirement_trait = self.resolve_trait(&type_name)?;
                    self.register_requirement(Rc::new(TraitBinding {
                        generic_to_type: HashMap::from([(Rc::clone(&requirement_trait.generics["Self"]), type_.clone())]),
                        trait_: requirement_trait,
                    }));
                }

                return Ok(type_)
            },
            _ => return Err(RuntimeError::new("Interpreted types aren't supported yet; please use an explicit type for now. 4".to_string())),
        }
    }
}
