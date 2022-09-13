use std::collections::{HashMap, HashSet};
use std::iter::zip;
use custom_error::custom_error;
use guard::guard;
use itertools::Itertools;
use uuid::Uuid;
use crate::program::types::{TypeUnit, Type, PassedArgumentType, FunctionInterface};

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
        return new
    }

    pub fn try_bind_alias(&mut self, generic: GenericAlias, t: &Type) -> Result<Box<Type>, TypeError> {
        let generic = self.register_alias(generic);
        self.try_bind(generic, t)
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
            // Two generics; merge rhs into lhs.
            (TypeUnit::Generic(lhs), TypeUnit::Generic(rhs)) => {
                let new_type = match (
                    self.alias_to_identity.get(lhs).and_then(|x| self.identity_to_type.remove(x)),
                    self.alias_to_identity.get(rhs).and_then(|x| self.identity_to_type.remove(x))
                ) {
                    // Two bound generics; merge them.
                    (Some(lhs), Some(rhs)) => Some(self.merge(&lhs, &rhs)?),
                    // One bound generic; use that one.
                    (Some(lhs), None) => Some(lhs.clone()),
                    (None, Some(rhs)) => Some(rhs.clone()),
                    // No bound generic; just merge aliases but bind no type.
                    (None, None) => None,
                };

                // If we have binding info, insert it into lhs' identity.
                if let Some(new_type) = new_type {
                    self.identity_to_type.insert(lhs.clone(), new_type);
                }

                // Remove rhs identity, and point alias towards lhs identity.
                let rhs_aliases = &self.identity_to_alias.remove(rhs).unwrap();
                self.identity_to_alias.get_mut(lhs).unwrap().extend(rhs_aliases);

                TypeUnit::Generic(lhs.clone())
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

        let arguments = zip(lhs.arguments.iter(), rhs.arguments.iter()).map(|(lhs, rhs)|
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

    pub fn merge_pairs(&mut self, pairs: &Vec<(&Type, &Type)>) -> Result<Vec<Box<Type>>, TypeError> {
        pairs.iter().map(|(lhs, rhs)| {
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
