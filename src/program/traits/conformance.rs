use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

use itertools::Itertools;

use crate::program::functions::FunctionHead;
use crate::program::traits::{Trait, TraitBinding};
use crate::program::types::TypeProto;
use crate::util::hash;

/// How a trait binding is fulfilled.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TraitConformance {
    /// The binding that is being fulfilled.
    pub binding: Rc<TraitBinding>,
    /// Mapping of: abstract function of the trait => the function that implements it.
    /// The functions have the same interfaces as the requirement (trait_.abstract_functions),
    ///  except with the generics replaced (binding.generic_to_type).
    pub function_mapping: HashMap<Rc<FunctionHead>, Rc<FunctionHead>>,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct TraitConformanceWithTail {
    /// The actual conformance.
    pub conformance: Rc<TraitConformance>,
    /// How the conformance was achieved (through dynamic rules).
    /// While the dynamic function call itself does not need this information, the dynamic dispatch (/monomorphization)
    ///  later needs to know more because the conformance's functions might call the tail's functions.
    /// You can imagine it like so: Suppose a function is called that has a requirement.
    /// Not only need the requirement be resolved - but if the conformance was achieved through rules, the implementations
    ///  of the requirements' functions might (will!) call functions from any of the rule's requirements!
    ///  e.g. Animal is required, the implementation is for any $Cat, so animal.talk() will call self.purr() - a function
    ///  declared only on Cats.
    /// So when we use this conformance, we must also bring along the tail
    /// which must be pre-resolved w.r.t. the conformance's requirements itself.
    pub tail: Rc<RequirementsFulfillment>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct RequirementsAssumption {
    pub conformance: HashMap<Rc<TraitBinding>, Rc<TraitConformance>>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct RequirementsFulfillment {
    // Requirement: (tail, conformance)
    pub conformance: HashMap<Rc<TraitBinding>, Rc<TraitConformanceWithTail>>,
    pub generic_mapping: HashMap<Rc<Trait>, Rc<TypeProto>>,
}


impl TraitConformance {
    pub fn new(binding: Rc<TraitBinding>, function_mapping: HashMap<Rc<FunctionHead>, Rc<FunctionHead>>,) -> Rc<TraitConformance> {
        Rc::new(TraitConformance {
            binding,
            function_mapping,
        })
    }

    pub fn pure(binding: Rc<TraitBinding>) -> Rc<TraitConformance> {
        if !binding.trait_.abstract_functions.is_empty() {
            panic!()
        }

        TraitConformance::new(binding, Default::default())
    }
}

impl Hash for TraitConformance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.binding.hash(state);
        for keyval in self.function_mapping.iter().sorted_by_key(|(src, dst)| src.function_id) {
            keyval.hash(state)
        }
    }
}

impl Hash for RequirementsFulfillment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (binding, conformance) in self.conformance.iter().sorted_by_key(|(binding, mapping)| hash::one(binding, DefaultHasher::new())) {
            binding.hash(state);
            conformance.hash(state);
        }

        for (id, type_) in self.generic_mapping.iter().sorted_by_key(|(trait_, type_)| trait_.id) {
            id.hash(state);
            type_.hash(state);
        }
    }
}


impl RequirementsFulfillment {
    pub fn empty() -> Rc<RequirementsFulfillment> {
        Rc::new(RequirementsFulfillment {
            conformance: Default::default(),
            generic_mapping: Default::default(),
        })
    }

    pub fn is_empty(&self) -> bool {
        self.conformance.is_empty() && self.generic_mapping.is_empty()
    }
}
