use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use uuid::Uuid;
use crate::error::{RResult, RuntimeError};
use crate::linker::scopes;
use crate::linker::scopes::Environment;
use crate::parser::ast;
use crate::program::generics::GenericAlias;
use crate::program::traits::{Trait, TraitBinding};
use crate::program::types::{TypeProto, TypeUnit};


pub struct TypeFactory<'a> {
    pub scope: &'a scopes::Scope<'a>,
    pub generics: HashMap<String, TypeUnit>,
    pub requirements: HashSet<Rc<TraitBinding>>,
}

impl <'a> TypeFactory<'a> {
    pub fn new(hierarchy: &'a scopes::Scope<'a>) -> TypeFactory<'a> {
        TypeFactory {
            scope: hierarchy,
            generics: HashMap::new(),
            requirements: HashSet::new(),
        }
    }

    fn resolve_reference(&mut self, name: &str) -> RResult<&TypeUnit> {
        if let Some(generic) = self.generics.get(name) {
            return Ok(generic)
        }

        self.scope.resolve(Environment::Global, &name)?.as_metatype()
    }

    fn resolve_trait(&mut self, name: &str) -> RResult<Rc<Trait>> {
        self.scope.resolve(Environment::Global, &name)?.as_trait()
    }

    fn register_generic(&mut self, name: &str, id: Uuid) -> &TypeUnit {
        self.generics.insert(name.to_string(), TypeUnit::Generic(id));
        self.generics.get(name).unwrap()
    }

    fn register_requirement(&mut self, requirement: Rc<TraitBinding>) {
        self.requirements.insert(requirement);
    }

    pub fn link_type(&mut self, syntax: &ast::Expression) -> RResult<Box<TypeProto>> {
        guard!(let Ok(pterm) = syntax.iter().exactly_one() else {
            panic!("Monads etc. are not implemented yet: '{}'", syntax)
        });
        let term: &ast::Term = &pterm.value;
        let arguments = vec![];

        match term {
            ast::Term::Identifier(type_name) => {
                match self.resolve_reference(type_name) {
                    Ok(unit) => {
                        Ok(Box::new(TypeProto {
                            unit: unit.clone(),
                            arguments
                        }))
                    }
                    Err(error) => {
                        if type_name.starts_with("#") || type_name.starts_with("$") {
                            let generic_id = GenericAlias::new_v4();
                            let type_ = Box::new(TypeProto {
                                unit: self.register_generic(type_name, generic_id).clone(),
                                arguments
                            });

                            if type_name.starts_with("$") {
                                let type_name = match type_name.find("#") {
                                    None => { String::from(&type_name[1..]) }
                                    Some(hash_start_index) => { String::from(&type_name[1..hash_start_index]) }
                                };

                                let requirement_trait = self.resolve_trait(&type_name)?;
                                self.register_requirement(Rc::new(TraitBinding {
                                    generic_to_type: HashMap::from([(requirement_trait.generics["Self"], type_.clone())]),
                                    trait_: requirement_trait,
                                }));
                            }

                            return Ok(type_)
                        }

                        Err(error)
                    }
                }
            },
            _ => panic!("Not a type!")
        }
    }

    pub fn generic_names(&self) -> HashMap<String, Uuid> {
        self.generics.iter().map(|(key, val)| (key.clone(), match val {
            TypeUnit::Generic(id) => *id,
            TypeUnit::Any(id) => *id,
            _ => panic!()
        })).collect()
    }
}
