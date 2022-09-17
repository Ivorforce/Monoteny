use std::collections::{HashMap, HashSet};
use custom_error::custom_error;
use guard::guard;
use itertools::{Itertools, zip_eq};
use uuid::Uuid;
use crate::program::types::{TypeUnit, Type};

pub type GenericIdentity = Uuid;
pub type GenericAlias = Uuid;

pub struct GenericMapping {
    pub identity_to_type: HashMap<GenericIdentity, Box<Type>>,
    pub alias_to_identity: HashMap<GenericAlias, GenericIdentity>,
    pub identity_to_alias: HashMap<GenericIdentity, HashSet<GenericAlias>>
}

custom_error!{pub TypeError
    MergeError{msg: String} = "Type Error: {msg}",
}

impl GenericMapping {
    pub fn new() -> GenericMapping {
        GenericMapping {
            identity_to_type: HashMap::new(),
            alias_to_identity: HashMap::new(),
            identity_to_alias: HashMap::new(),
        }
    }

    //  ----- alias

    pub fn make_generic_alias_type(&mut self) -> Box<Type> {
         let id = Uuid::new_v4();
        self.register_alias(id.clone());

        return Type::unit(TypeUnit::Generic(id))
    }

    pub fn register_alias(&mut self, alias: GenericAlias) -> GenericIdentity {
        if let Some(existing) = self.alias_to_identity.get_mut(&alias) {
            return existing.clone()
        }

        let new = Uuid::new_v4();
        self.alias_to_identity.insert(alias, new.clone());
        self.identity_to_alias.insert(new.clone(), HashSet::from([alias.clone()]));
        return new
    }

    pub fn try_bind_alias(&mut self, generic: GenericAlias, t: &Type) -> Result<Box<Type>, TypeError> {
        let generic = self.register_alias(generic);
        self.try_bind(generic, t)
    }

    pub fn resolve_type(&self, type_: &Box<Type>) -> Result<Box<Type>, TypeError> {
        match &type_.unit {
            TypeUnit::Generic(alias) => self.resolve_binding_alias(alias).map(|x| x.clone()),
            _ => Ok(Box::new(Type {
                unit: type_.unit.clone(),
                arguments: type_.arguments.iter().map(|x| self.resolve_type(x)).try_collect()?
            }))
        }
    }

    pub fn resolve_binding_alias(&self, alias: &GenericAlias) -> Result<&Box<Type>, TypeError> {
        guard!(let Some(identity) = self.alias_to_identity.get(alias) else {
            return Err(TypeError::MergeError { msg: format!("Unknown generic: {}", alias) })
        });

        guard!(let Some(binding) = self.identity_to_type.get(identity) else {
            return Err(TypeError::MergeError { msg: format!("Unbound generic: {}", alias) })
        });

        return Ok(binding)
    }

    //  ----- non-alias

    pub fn try_bind(&mut self, identity: GenericIdentity, t: &Type) -> Result<Box<Type>, TypeError> {
        if let Some(existing) = self.identity_to_type.remove(&identity) {
            let resolved_type = self.merge(&existing, t)?;

            self.identity_to_type.insert(identity, resolved_type.clone());
            Ok(resolved_type)
        }
        else {
            self.identity_to_type.insert(identity, Box::new(t.clone()));
            Ok(Box::new(t.clone()))
        }
    }

    pub fn merge(&mut self, lhs: &Type, rhs: &Type) -> Result<Box<Type>, TypeError> {
        let unit: TypeUnit = match (&lhs.unit, &rhs.unit) {
            // Two generics; merge into lhs.
            (TypeUnit::Generic(lhs_alias), TypeUnit::Generic(rhs_alias)) => {
                // TODO What's the best way to do this, especially if we don't know if lhs or rhs are bound?
                let lhs = &self.register_alias(lhs_alias.clone());

                if let Some(rhs) = self.alias_to_identity.get(rhs_alias).map(Clone::clone) {
                    // rhs exists too; need to merge

                    match (
                        // Have to remove both so we can call self.merge() without cloning the types.
                        self.identity_to_type.remove(lhs),
                        self.identity_to_type.remove(&rhs)
                    ) {
                        // Two bound generics; use lhs and bind the merge result
                        (Some(lhs_type), Some(rhs_type)) => {
                            let merged_type = self.merge(&lhs_type, &rhs_type)?;
                            self.identity_to_type.insert(lhs.clone(), merged_type);
                        }
                        // Just lhs was bound; bind to lhs
                        (Some(lhs_type), None) => {
                            self.identity_to_type.insert(lhs.clone(), lhs_type);
                        },
                        // Just rhs was bound; bind to lhs
                        (None, Some(rhs_type)) => {
                            self.identity_to_type.insert(lhs.clone(), rhs_type);
                        },
                        // No bound generic
                        (None, None) => {},
                    }

                    // Merge rhs aliases into lhs identity
                    let aliases = self.identity_to_alias.remove(&rhs).unwrap();
                    for alias in aliases.iter() {
                        self.alias_to_identity.insert(alias.clone(), lhs.clone());
                    }
                    self.identity_to_alias.get_mut(lhs).unwrap().extend(aliases);
                }
                else {
                    // Register rhs alias into lhs identity
                    self.identity_to_alias.get_mut(lhs).unwrap().insert(rhs_alias.clone());
                    self.alias_to_identity.insert(rhs_alias.clone(), lhs.clone());
                }

                // Return lhs alias as generic
                TypeUnit::Generic(lhs_alias.clone())
            },
            // Just one generic; use that and merge the other into it.
            (TypeUnit::Generic(lhs), _) => {
                self.try_bind_alias(lhs.clone(), rhs)?;
                TypeUnit::Generic(lhs.clone())
            },
            (_, TypeUnit::Generic(rhs)) => {
                self.try_bind_alias(rhs.clone(), lhs)?;
                TypeUnit::Generic(rhs.clone())
            },
            // No generic; just merge the two.
            (lhs, rhs) => {
                if lhs != rhs {
                    // TODO Print alias names instead.
                    return Err(TypeError::MergeError { msg: format!("Generics {:?} and {:?} are not compatible: binding", lhs, rhs) })
                }

                lhs.clone()
            }
        };

        // Now, merge all arguments.
        if lhs.arguments.len() != rhs.arguments.len() {
            return Err(TypeError::MergeError { msg: format!("Types {:?} and {:?} are not compatible: argument count", lhs, rhs) })
        }

        let arguments = zip_eq(lhs.arguments.iter(), rhs.arguments.iter()).map(|(lhs, rhs)|
            self.merge(lhs, rhs)
        ).try_collect()?;

        return Ok(Box::new(Type { unit, arguments }))
    }

    pub fn merge_all<'a>(&mut self, types: &Vec<&'a Type>) -> Result<Box<Type>, TypeError> {
        if types.is_empty() {
            // No elements, so we can be whatever we want to be!
            return Ok(self.make_generic_alias_type())
        }

        let mut reference = Box::new(types[0].clone());
        for other in types.iter().skip(1) {
            reference = self.merge(&reference, other)?;
        }

        return Ok(reference)
    }

    pub fn merge_pairs<'a, I>(&mut self, pairs: I) -> Result<Vec<Box<Type>>, TypeError> where I: Iterator<Item=(&'a Type, &'a Type)> {
        pairs.map(|(lhs, rhs)| {
            self.merge(lhs, rhs)
        }).try_collect()
    }
}

impl Clone for GenericMapping {
    fn clone(&self) -> Self {
        GenericMapping {
            identity_to_type: self.identity_to_type.clone(),
            alias_to_identity: self.alias_to_identity.clone(),
            identity_to_alias: self.identity_to_alias.clone(),
        }
    }
}
