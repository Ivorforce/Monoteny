use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use itertools::{zip_eq, Itertools};
use uuid::Uuid;

use crate::error::{RResult, RuntimeError, TryCollectMany};
use crate::program::traits::Trait;
use crate::program::types::{TypeProto, TypeUnit};

pub type GenericIdentity = Uuid;
pub type GenericAlias = Uuid;

#[derive(Clone)]
pub struct TypeForest {
    /// From internal identities to type units.
    /// Cannot contain TypeUnit::Generic because then the identity's users
    /// should have pointed to that generics' identity instead.
    pub identity_to_type: HashMap<GenericIdentity, TypeUnit>,
    pub identity_to_arguments: HashMap<GenericIdentity, Vec<GenericIdentity>>,

    pub alias_to_identity: HashMap<GenericAlias, GenericIdentity>,
    pub identity_to_alias: HashMap<GenericIdentity, HashSet<GenericAlias>>,
}

impl TypeForest {
    pub fn new() -> TypeForest {
        TypeForest {
            identity_to_type: HashMap::new(),
            identity_to_arguments: HashMap::new(),
            alias_to_identity: HashMap::new(),
            identity_to_alias: HashMap::new(),
        }
    }

    //  ----- alias

    pub fn register(&mut self, alias: GenericAlias) {
        self._register(alias);
    }

    pub fn bind(&mut self, generic: GenericAlias, t: &TypeProto) -> RResult<()> {
        let identity = self._register(generic);
        self.bind_identity(identity, t)
    }

    pub fn is_bound_to(&self, generic: &GenericAlias, t: &TypeProto) -> bool {
        self.is_identity_bound_to(self.alias_to_identity.get(generic).unwrap(), t)
    }

    pub fn get_unit(&self, generic: &GenericAlias) -> Option<&TypeUnit> {
        let identity = self.alias_to_identity.get(generic).unwrap();
        self.identity_to_type.get(identity)
    }

    pub fn resolve_type(&self, type_: &TypeProto) -> RResult<Rc<TypeProto>> {
        match &type_.unit {
            TypeUnit::Generic(alias) => self.resolve_binding_alias(alias).map(|x| x.clone()),
            _ => Ok(Rc::new(TypeProto {
                unit: type_.unit.clone(),
                arguments: type_.arguments.iter().map(|x| self.resolve_type(x)).try_collect_many()?
            }))
        }
    }

    pub fn resolve_binding_alias(&self, alias: &GenericAlias) -> RResult<Rc<TypeProto>> {
        let Some(identity) = self.alias_to_identity.get(alias) else {
            return Err(RuntimeError::error(format!("Unknown generic: {}", alias).as_str()).to_array())
        };

        let Some(binding) = self.identity_to_type.get(identity) else {
            return Ok(TypeProto::unit(TypeUnit::Generic(*alias)))
        };

        return Ok(Rc::new(TypeProto {
            unit: binding.clone(),
            arguments: self.identity_to_arguments.get(&identity).unwrap().iter()
                .map(|x| self.resolve_binding_alias(x))
                .try_collect_many()?
        }))
    }

    pub fn prototype_binding_alias(&self, alias: &GenericAlias) -> Rc<TypeProto> {
        let Some(identity) = self.alias_to_identity.get(alias) else {
            return TypeProto::unit(TypeUnit::Generic(*alias));
        };

        let Some(binding) = self.identity_to_type.get(identity) else {
            return TypeProto::unit(TypeUnit::Generic(*alias));
        };

        return Rc::new(TypeProto {
            unit: binding.clone(),
            arguments: self.identity_to_arguments.get(&identity).unwrap().iter()
                .map(|x| self.prototype_binding_alias(x))
                .collect()
        })
    }

    pub fn merge_all(&mut self, types: &Vec<GenericAlias>) -> RResult<GenericAlias> {
        if types.is_empty() {
            // No elements, so we can be whatever we want to be!
            let id = Uuid::new_v4();
            self.register(id.clone());
            return Ok(id)
        }

        let reference = types[0].clone();
        for other in types.iter().skip(1) {
            self.merge_identities(reference.clone(), other.clone())?;
        }

        return Ok(reference)
    }

    pub fn rebind(&mut self, generic: GenericAlias, t: &TypeProto) -> RResult<()> {
        let Some(identity) = self.alias_to_identity.get(&generic) else {
            panic!("Internal Error: Cannot rebind non existing generic ({}), aborting.", generic);
        };

        self.identity_to_type.remove(identity);
        self.bind_identity(*identity, t)
    }

    pub fn rebind_structs_as_generic(&mut self, structs: &HashMap<Rc<Trait>, Rc<TypeProto>>) -> RResult<()>{
        let map: HashMap<_, _> = structs.into_iter().map(|(struct_, type_)| {
            let identity = self._register(struct_.id);
            self.bind_identity(identity, type_)?;
            Ok::<(&Rc<Trait>, Uuid), Vec<RuntimeError>>((struct_, identity))
        }).try_collect_many()?;

        let mut replace_map = HashMap::new();
        for (other_identity, unit) in self.identity_to_type.iter() {
            if let TypeUnit::Struct(struct_) = unit {
                if let Some(target_identity) = map.get(struct_) {
                    replace_map.insert(*other_identity, *target_identity);
                }
            }
        }
        for (other_identity, target_identity) in replace_map {
            self.relink_identity(other_identity, target_identity);
        }

        Ok(())
    }

    //  ----- non-alias

    fn _register(&mut self, alias: GenericAlias) -> GenericIdentity {
        if let Some(existing) = self.alias_to_identity.get(&alias) {
            return existing.clone()
        }

        let new = Uuid::new_v4();
        self.alias_to_identity.insert(alias, new.clone());
        self.identity_to_alias.insert(new.clone(), HashSet::from([alias.clone()]));
        return new
    }

    fn bind_identity(&mut self, identity: GenericIdentity, t: &TypeProto) -> RResult<()> {
        // TODO This could be done faster by not creating a new ID,
        //  but for now this approach saves us boilerplate / duplicate code.
        let new_id = self.insert_new_identity(t);
        self.merge_identities(identity, new_id)?;
        Ok(())
    }

    fn insert_new_identity(&mut self, t: &TypeProto) -> GenericIdentity {
        match &t.unit {
            TypeUnit::Generic(alias) => {
                // If the generic already has an identity, return that. Otherwise, make one.
                self._register(alias.clone())
            },
            _ => {
                let identity = GenericIdentity::new_v4();
                self.identity_to_type.insert(identity.clone(), t.unit.clone());
                self.identity_to_alias.insert(identity.clone(), HashSet::new());

                let arguments = t.arguments.iter()
                    .map(|arg| self.insert_new_identity(arg))
                    .collect();

                self.identity_to_arguments.insert(identity, arguments);

                identity
            }
        }
    }

    fn is_identity_bound_to(&self, id: &GenericIdentity, t: &TypeProto) -> bool {
        match self.identity_to_type.get(id) {
            None => return false,
            Some(bound) => {
                if &t.unit != bound {
                    return false;
                }
            }
        }

        let args = self.identity_to_arguments.get(id).unwrap();

        if args.len() != t.arguments.len() {
            return false;
        }

        for (bound_arg, arg) in zip_eq(args, t.arguments.iter()) {
            if !self.is_identity_bound_to(bound_arg, arg) {
                return false;
            }
        }

        return true;
    }

    fn merge_identities(&mut self, lhs: GenericIdentity, rhs: GenericIdentity) -> RResult<GenericIdentity> {
        if lhs == rhs {
            return Ok(lhs)
        }

        // TODO We default to "into lhs" out of convenience, but it may be faster to use rhs sometimes
        // Merge rhs aliases / arguments into lhs
        self.relink_identity(rhs, lhs);

        // Merge types
        let rhs_type = self.identity_to_type.remove(&rhs);
        match (self.identity_to_type.entry(lhs), rhs_type) {
            (Entry::Occupied(lhs_entry), Some(rhs_type)) => {
                // Need to merge.
                if lhs_entry.get() != &rhs_type {
                    return Err(RuntimeError::error(format!("Cannot merge types: {:?} and {:?}", lhs_entry.get(), rhs_type).as_str()).to_array())
                }

                // TODO This might fall into a trap of recursion circles
                // Merge arguments one by one.
                for (arg, r_arg) in zip_eq(
                    self.identity_to_arguments.get(&lhs).unwrap().clone(),
                    self.identity_to_arguments.remove(&rhs).unwrap()
                ) {
                    self.merge_identities(arg, r_arg)?;
                }
            }
            (Entry::Vacant(lhs_entry), Some(rhs_type)) => {
                // No left entry; we can just move right into left.
                lhs_entry.insert(rhs_type);
                let rhs_args = self.identity_to_arguments.remove(&rhs).unwrap();
                self.identity_to_arguments.insert(lhs, rhs_args);
            }
            (_, None) => {}  // Nothing to merge, right is empty.
        }

        Ok(lhs)
    }

    fn relink_identity(&mut self, source: GenericIdentity, target: GenericIdentity) {
        // TODO This is pretty naive; maybe we also want a reverse map here too?
        for args in self.identity_to_arguments.values_mut() {
            if args.contains(&source) {
                *args = args.iter().map(|x| if *x == source { target } else { *x } ).collect();
            }
        }

        let source_aliases = self.identity_to_alias.remove(&source).unwrap();
        for alias in source_aliases.iter() {
            self.alias_to_identity.insert(alias.clone(), target);
        }
        self.identity_to_alias.get_mut(&target).unwrap().extend(source_aliases);
    }
}
