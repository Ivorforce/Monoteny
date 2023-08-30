use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::rc::Rc;
use uuid::Uuid;
use crate::linker::{LinkError, scopes};
use crate::linker::scopes::Environment;
use crate::parser::abstract_syntax;
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

    fn resolve_reference(&mut self, name: &String) -> Result<&TypeUnit, LinkError> {
        if let Some(generic) = self.generics.get(name) {
            return Ok(generic)
        }

        self.scope.resolve(Environment::Global, &name)?.as_metatype()
    }

    fn resolve_trait(&mut self, name: &String) -> Rc<Trait> {
        self.scope.resolve(Environment::Global, &name).unwrap().as_trait().unwrap()
    }

    fn register_generic(&mut self, name: &String, id: Uuid) -> &TypeUnit {
        self.generics.insert(name.clone(), TypeUnit::Generic(id));
        self.generics.get(name).unwrap()
    }

    fn register_requirement(&mut self, requirement: Rc<TraitBinding>) {
        self.requirements.insert(requirement);
    }

    pub fn link_type(&mut self, syntax: &abstract_syntax::Expression) -> Result<Box<TypeProto>, LinkError> {
        if syntax.len() > 1 {
            panic!("Monads etc. are not implemented yet!")
        }

        let arguments = vec![];

        match syntax.iter().next().unwrap().as_ref() {
            abstract_syntax::Term::Identifier(type_name) => {
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

                                let requirement_trait = self.resolve_trait(&type_name);
                                self.register_requirement(Rc::new(TraitBinding {
                                    generic_to_type: HashMap::from([(requirement_trait.generics["self"], type_.clone())]),
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
}
