use std::collections::{HashMap, HashSet};
use guard::guard;
use itertools::{Itertools, zip_eq};
use uuid::Uuid;
use crate::error::RuntimeError;
use crate::program::types::{TypeUnit, TypeProto};

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

    pub fn bind(&mut self, generic: GenericAlias, t: &TypeProto) -> Result<(), RuntimeError> {
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

    pub fn resolve_type(&self, type_: &TypeProto) -> Result<Box<TypeProto>, RuntimeError> {
        match &type_.unit {
            TypeUnit::Generic(alias) => self.resolve_binding_alias(alias).map(|x| x.clone()),
            _ => Ok(Box::new(TypeProto {
                unit: type_.unit.clone(),
                arguments: type_.arguments.iter().map(|x| self.resolve_type(x)).try_collect()?
            }))
        }
    }

    pub fn resolve_binding_alias(&self, alias: &GenericAlias) -> Result<Box<TypeProto>, RuntimeError> {
        guard!(let Some(identity) = self.alias_to_identity.get(alias) else {
            return Err(RuntimeError { msg: format!("Unknown generic: {}", alias) })
        });

        guard!(let Some(binding) = self.identity_to_type.get(identity) else {
            return Ok(Box::new(*TypeProto::unit(TypeUnit::Generic(*alias))))
        });

        return Ok(Box::new(TypeProto {
            unit: binding.clone(),
            arguments: self.identity_to_arguments.get(&identity).unwrap().iter()
                .map(|x| self.resolve_binding_alias(x))
                .try_collect()?
        }))
    }

    pub fn prototype_binding_alias(&self, alias: &GenericAlias) -> Box<TypeProto> {
        guard!(let Some(identity) = self.alias_to_identity.get(alias) else {
            return TypeProto::unit(TypeUnit::Generic(*alias));
        });

        guard!(let Some(binding) = self.identity_to_type.get(identity) else {
            return TypeProto::unit(TypeUnit::Generic(*alias));
        });

        return Box::new(TypeProto {
            unit: binding.clone(),
            arguments: self.identity_to_arguments.get(&identity).unwrap().iter()
                .map(|x| self.prototype_binding_alias(x))
                .collect()
        })
    }

    pub fn merge_all(&mut self, types: &Vec<GenericAlias>) -> Result<GenericAlias, RuntimeError> {
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

    pub fn rebind(&mut self, generic: GenericAlias, t: &TypeProto) -> Result<(), RuntimeError> {
        guard!(let Some(identity) = self.alias_to_identity.get(&generic) else {
            panic!("Internal Error: Cannot rebind non existing generic ({}), aborting.", generic);
        });

        self.identity_to_type.remove(identity);
        self.bind_identity(*identity, t)
    }

    pub fn bind_any_as_generic(&mut self, anys: &HashMap<GenericAlias, Box<TypeProto>>) -> Result<(), RuntimeError>{
        let map: HashMap<_, _> = anys.into_iter().map(|(any, type_)| {
            let identity = self._register(*any);
            self.bind_identity(identity, type_)?;
            Ok((any, identity))
        }).try_collect()?;

        let mut replace_map = HashMap::new();
        for (other_identity, unit) in self.identity_to_type.iter() {
            if let TypeUnit::Any(any) = unit {
                if let Some(target_identity) = map.get(any) {
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

    fn bind_identity(&mut self, identity: GenericIdentity, t: &TypeProto) -> Result<(), RuntimeError> {
        // TODO This could be done faster by not creating a new ID, but for now this approach saves us boilerplate / duplicate code
        let new_id = self.insert_new_identity(t);
        self.merge_identities(identity, new_id)?;
        Ok(())
    }

    fn insert_new_identity(&mut self, t: &TypeProto) -> GenericIdentity {
        match &t.unit {
            TypeUnit::Generic(alias) => {
                // May already exist, but since the proto has no bind for it, we need not try to further bind.
                // Either way whatever the register returns is already correct.
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

    fn merge_identities(&mut self, lhs: GenericIdentity, rhs: GenericIdentity) -> Result<GenericIdentity, RuntimeError> {
        // Merge rhs aliases / arguments into lhs
        self.relink_identity(rhs, lhs);

        // Merge types
        // TODO This might be faster if we first check whether it's better to merge into lhs or into rhs.
        match (self.identity_to_type.remove(&lhs), self.identity_to_type.remove(&rhs)) {
            (Some(lhs_type), Some(rhs_type)) => {
                if lhs_type != rhs_type {
                    return Err(RuntimeError { msg: format!("Cannot merge types: {:?} and {:?}", lhs_type, rhs_type) })
                }
                self.identity_to_type.insert(lhs.clone(), lhs_type);

                let lhs_args: Vec<GenericIdentity> = self.identity_to_arguments.get(&lhs).unwrap().iter().map(Clone::clone).collect();

                // TODO This might fall into a trap of recursion circles
                // Merge arguments
                for (arg, r_arg) in zip_eq(
                    lhs_args,
                    self.identity_to_arguments.remove(&rhs).unwrap()
                ) {
                    self.merge_identities(arg, r_arg)?;
                }
            }
            (Some(lhs_type), None) => {
                self.identity_to_type.insert(lhs.clone(), lhs_type);
            }
            (None, Some(rhs_type)) => {
                self.identity_to_type.insert(lhs.clone(), rhs_type);
                let rhs_args = self.identity_to_arguments.remove(&rhs).unwrap();
                self.identity_to_arguments.insert(lhs, rhs_args);
            }
            (None, None) => {}  // Nothing to bind
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

        let rhs_aliases = self.identity_to_alias.remove(&source).unwrap();
        for alias in rhs_aliases.iter() {
            self.alias_to_identity.insert(alias.clone(), target);
        }
        self.identity_to_alias.get_mut(&target).unwrap().extend(rhs_aliases);
    }
}
