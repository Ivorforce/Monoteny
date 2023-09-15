use std::collections::{HashMap, HashSet};
use std::collections::hash_map::{DefaultHasher, Entry};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use uuid::Uuid;
use crate::linker::LinkError;
use crate::program::functions::{FunctionHead, FunctionPointer, FunctionType, FunctionInterface};
use crate::program::generics::{GenericAlias, TypeForest};
use crate::program::types::{TypeProto, TypeUnit};
use crate::util::fmt::write_keyval;
use crate::util::hash;

/// The definition of some trait.
#[derive(Clone)]
pub struct Trait {
    pub id: Uuid,
    pub name: String,

    // Generics declared for this trait, by name (via its declaration).
    // Used in abstract functions and requirements (collect_generics on those would yield the same GenericAliases).
    pub generics: HashMap<String, GenericAlias>,

    // To conform to this trait, these other conformances are required.
    pub requirements: HashSet<Rc<TraitBinding>>,

    // Functions required by this trait specifically (not its requirements).
    // The head of each function to its pointer (how it is defined).
    pub abstract_functions: HashMap<Rc<FunctionHead>, Rc<FunctionPointer>>,
}

/// Some application of a trait with specific types.
#[derive(Clone, Eq, PartialEq)]
pub struct TraitBinding {
    /// The trait that is bound.
    pub trait_: Rc<Trait>,

    /// A mapping from each of the trait's generics to some type.
    pub generic_to_type: HashMap<Uuid, Box<TypeProto>>,
}

/// How a trait binding is fulfilled.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TraitConformance {
    // The binding that is being fulfilled.
    pub binding: Rc<TraitBinding>,
    // abstract function of the trait to the function that implements it.
    pub function_mapping: HashMap<Rc<FunctionHead>, Rc<FunctionHead>>,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct TraitConformanceWithTail {
    pub conformance: Rc<TraitConformance>,
    pub tail: Rc<RequirementsFulfillment>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TraitConformanceRule {
    // Generics declared for this conformance, by name (via its declaration).
    // Used in requirements and the conformance itself (collect_generics on those would yield the same GenericAliases).
    pub generics: HashMap<String, GenericAlias>,

    // To use this conformance, these other conformances are required.
    pub requirements: HashSet<Rc<TraitBinding>>,

    // The conformance (w.r.t. generics) defined by this rule.
    pub conformance: Rc<TraitConformance>,
}

/// A sum of knowledge about trait conformance.
/// You can query this to find out if some binding can be cast to some other binding.
/// It caches conformance for subtraits so that lookup is fast.
#[derive(Clone, Eq, PartialEq)]
pub struct TraitGraph {
    /// All known conformances.
    /// For each conformance, we also know its tail - aka HOW the conformance was achieved (through dynamic rules).
    /// While the dynamic function call itself does not need this information, the dynamic dispatch (/monomorphization)
    ///  later needs to know more because the conformance's functions might call the tail's functions.
    /// You can imagine it like so: Suppose a function is called that has a requirement.
    /// Not only need the requirement be resolved - but if the requirement was achieved through rules, the implementations
    ///  of the requirements' functions might (will!) call functions from any of the rule's requirements!
    ///  e.g. Animal is required, the implementation is for any $Cat, so animal.talk() will call self.purr() - a function
    ///  declared only on Cats. So when we use this conformance, we must also bring along the tail, which must be pre-resolved
    ///  w.r.t. the conformance's requirements itself.
    pub conformance_cache: HashMap<Rc<TraitBinding>, Option<Rc<TraitConformanceWithTail>>>,

    /// A list of conformance declarations that allow for dynamic conformance.
    /// All these use generics in the conformance, which are provided by the requirements.
    /// To use the conformance, these generics should be replaced by the matching bindings.
    pub conformance_rules: HashMap<Rc<Trait>, Vec<Rc<TraitConformanceRule>>>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct RequirementsAssumption {
    pub conformance: HashMap<Rc<TraitBinding>, Rc<TraitConformance>>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct RequirementsFulfillment {
    // Requirement: (tail, conformance)
    pub conformance: HashMap<Rc<TraitBinding>, Rc<TraitConformanceWithTail>>,
    pub generic_mapping: HashMap<Uuid, Box<TypeProto>>,
}

impl TraitGraph {
    pub fn new() -> TraitGraph {
        TraitGraph {
            conformance_cache: Default::default(),
            conformance_rules: Default::default(),
        }
    }

    pub fn clear_cache(&mut self) {
        self.conformance_cache = HashMap::new();
    }

    pub fn add_graph(&mut self, graph: &TraitGraph) {
        self.conformance_cache.clear();
        for (trait_, rules) in graph.conformance_rules.iter() {
            match self.conformance_rules.entry(Rc::clone(trait_)) {
                Entry::Occupied(o) => _ = o.into_mut().extend(rules.clone()),
                Entry::Vacant(v) => _ = v.insert(rules.clone()),
            }
        }
    }

    pub fn add_conformance_rule(&mut self, rule: Rc<TraitConformanceRule>) {
        match self.conformance_rules.entry(Rc::clone(&rule.conformance.binding.trait_)) {
            Entry::Occupied(e) => {
                e.into_mut().push(rule);
            }
            Entry::Vacant(e) => {
                e.insert(vec![rule]);
            }
        };
    }

    pub fn satisfy_requirement(&mut self, requirement: &Rc<TraitBinding>, mapping: &TypeForest) -> Result<Rc<TraitConformanceWithTail>, LinkError> {
        // TODO What if requirement is e.g. Float<Float>? Is Float declared on itself?

        // We resolve this binding because it might contain generics.
        let resolved_binding = requirement.try_mapping_types(&|type_| mapping.resolve_type(type_))?;
        if !resolved_binding.collect_generics().is_empty() {
            return Err(LinkError::Ambiguous);
        }

        if let Some(state) = self.conformance_cache.get(&resolved_binding) {
            // In cache
            return match state {
                None => Err(LinkError::LinkError { msg: String::from(format!("No compatible declaration for trait conformance requirement: {:?}", resolved_binding)) }),
                Some(declaration) => Ok(declaration.clone()),
            };
        }

        guard!(let Some(relevant_declarations) = self.conformance_rules.get(&resolved_binding.trait_) else {
            return Err(LinkError::LinkError { msg: String::from(format!("No declarations found for trait: {:?}", resolved_binding.trait_)) });
        });

        // Recalculate
        // TODO clone is a bit much, but we need it to be memory safe
        let cloned_declarations: Vec<Rc<TraitConformanceRule>> = relevant_declarations.clone();
        'rule: for rule in cloned_declarations.iter() {
            // TODO The type forest should be able to do this for us. Do we really have to do both?
            guard!(let Some(generics_map) = TraitBinding::merge(&rule.conformance.binding, &resolved_binding) else {
                continue;
            });

            // We have to make a new type forest so that the fulfillment returned by test_requirements uses generics.
            let mut rule_mapping = mapping.clone();
            for (generic, type_) in generics_map.iter() {
                if rule_mapping.bind(*generic, type_).is_err() {
                    continue 'rule;
                }
            }

            if let Ok(fulfilled_requirements) = self.test_requirements(&rule.requirements, &rule_mapping) {
                let resolved_conformance = TraitConformance::new(
                    resolved_binding.clone(),
                    // TODO Do we need to map the functions?
                    rule.conformance.function_mapping.clone(),
                );
                // TODO There may be more than one conflicting solution
                let pair = Rc::new(TraitConformanceWithTail {
                    tail: Rc::new(RequirementsFulfillment {
                        conformance: fulfilled_requirements,
                        generic_mapping: generics_map.clone(),
                    }),
                    conformance: resolved_conformance,
                });
                self.conformance_cache.insert(resolved_binding, Some(pair.clone()));
                return Ok(pair.clone());
            }
        }

        self.conformance_cache.insert(Rc::clone(&resolved_binding), None);
        Err(LinkError::LinkError { msg: String::from(format!("No compatible declaration for trait conformance requirement: {:?}. {} rules failed the check: {:?}", resolved_binding, cloned_declarations.len(), cloned_declarations)) })
    }

    pub fn test_requirements(&mut self, requirements: &HashSet<Rc<TraitBinding>>, mapping: &TypeForest) -> Result<HashMap<Rc<TraitBinding>, Rc<TraitConformanceWithTail>>, LinkError> {
        let mut conformance = HashMap::new();

        for requirement in self.gather_deep_requirements(requirements.iter().cloned()) {
            let trait_conformance = self.satisfy_requirement(&requirement, &mapping)?;
            conformance.insert(requirement.clone(), trait_conformance);
        }

        Ok(conformance)
    }

    pub fn gather_deep_requirements<C>(&self, bindings: C) -> Vec<Rc<TraitBinding>> where C: Iterator<Item=Rc<TraitBinding>> {
        let mut all = HashSet::new();
        let mut ordered = vec![];
        let mut rest = bindings.collect_vec();
        while let Some(binding) = rest.pop() {
            if all.insert(Rc::clone(&binding)) {
                ordered.push(Rc::clone(&binding));
                rest.extend(
                    binding.trait_.requirements.iter()
                        .map(|x| x.mapping_types(&|type_| type_.replacing_generics(&binding.generic_to_type))))
            }
        }
        ordered.reverse();
        ordered
    }

    pub fn assume_granted<C>(&self, bindings: C) -> Vec<Rc<TraitConformance>> where C: Iterator<Item=Rc<TraitBinding>> {
        let deep_requirements = self.gather_deep_requirements(bindings);
        let mut resolutions = vec![];

        for trait_binding in deep_requirements.iter() {
            let mut binding_resolution = HashMap::new();

            for abstract_fun in trait_binding.trait_.abstract_functions.values() {
                let mapped_pointer = Rc::new(FunctionPointer {
                    name: abstract_fun.name.clone(),
                    form: abstract_fun.form.clone(),
                    target: FunctionHead::new(
                        Rc::new(FunctionInterface {
                            parameters: abstract_fun.target.interface.parameters.iter().map(|x| {
                                x.mapping_type(&|type_| type_.replacing_generics(&trait_binding.generic_to_type))
                            }).collect(),
                            return_type: abstract_fun.target.interface.return_type.replacing_generics(&trait_binding.generic_to_type),
                            requirements: abstract_fun.target.interface.requirements.iter().map(|req| {
                                req.mapping_types(&|type_| type_.replacing_generics(&trait_binding.generic_to_type))
                            }).collect(),
                        }),
                        FunctionType::Polymorphic {
                            provided_by_assumption: Rc::clone(&trait_binding),
                            abstract_function: Rc::clone(abstract_fun)
                        }
                    ),
                });
                binding_resolution.insert(
                    Rc::clone(&abstract_fun.target),
                    Rc::clone(&mapped_pointer.target)
                );
            }

            resolutions.push(
                TraitConformance::new(Rc::clone(trait_binding), binding_resolution)
            );
        }

        resolutions
    }
}

impl Trait {
    pub fn new_with_self(name: String) -> Trait {
        Trait {
            id: Uuid::new_v4(),
            name,
            generics: HashMap::from([("self".to_string(), Uuid::new_v4())]),
            requirements: Default::default(),
            abstract_functions: Default::default(),
        }
    }

    pub fn create_generic_type(self: &Trait, generic_name: &str) -> Box<TypeProto> {
        TypeProto::unit(TypeUnit::Generic(self.generics[generic_name]))
    }

    pub fn create_generic_binding(self: &Rc<Trait>, generic_to_type: Vec<(&str, Box<TypeProto>)>) -> Rc<TraitBinding> {
        Rc::new(TraitBinding {
            trait_: Rc::clone(self),
            generic_to_type: HashMap::from_iter(
                generic_to_type.into_iter()
                    .map(|(generic_name, type_)| (self.generics[generic_name], type_))
            ),
        })
    }

    pub fn insert_function(&mut self, function: Rc<FunctionPointer>) {
        self.abstract_functions.insert(Rc::clone(&function.target), function);
    }

    pub fn insert_functions<'a, I>(&mut self, functions: I) where I: Iterator<Item=&'a Rc<FunctionPointer>> {
        for ptr in functions {
            self.insert_function(Rc::clone(ptr))
        }
    }

    pub fn add_simple_parent_requirement(&mut self, parent_trait: &Rc<Trait>) {
        self.requirements.insert(
            parent_trait.create_generic_binding(vec![("self", self.create_generic_type("self"))])
        );
    }
}

impl TraitBinding {
    pub fn mapping_types(&self, map: &dyn Fn(&Box<TypeProto>) -> Box<TypeProto>) -> Rc<TraitBinding> {
        Rc::new(TraitBinding {
            trait_: Rc::clone(&self.trait_),
            generic_to_type: self.generic_to_type.iter().map(|(generic_id, type_) | (*generic_id, map(type_))).collect()
        })
    }

    pub fn try_mapping_types<B>(&self, map: &dyn Fn(&Box<TypeProto>) -> Result<Box<TypeProto>, B>) -> Result<Rc<TraitBinding>, B> {
        Ok(Rc::new(TraitBinding {
            trait_: Rc::clone(&self.trait_),
            generic_to_type: self.generic_to_type.iter().map(|(generic_id, type_) | Ok((*generic_id, map(type_)?))).try_collect()?
        }))
    }

    pub fn collect_generics(&self) -> HashSet<GenericAlias> {
        TypeProto::collect_generics(self.generic_to_type.values())
    }

    /// Merge the two bindings. This will return None if the merger fails.
    /// If it succeeds, it returns a map of generic to type that was resolved during the merger.
    pub fn merge(lhs: &TraitBinding, rhs: &TraitBinding) -> Option<HashMap<Uuid, Box<TypeProto>>> {
        if lhs.trait_ != rhs.trait_ || lhs.generic_to_type.keys().collect::<HashSet<_>>() != rhs.generic_to_type.keys().collect() {
            return None
        }

        let mut types = TypeForest::new();
        for (key, type_) in lhs.generic_to_type.iter() {
            types.bind(*key, type_).ok()?;
        }
        for (key, type_) in rhs.generic_to_type.iter() {
            types.bind(*key, type_).ok()?;
        }
        Some(
            lhs.collect_generics().union(&rhs.collect_generics()).into_iter()
                .map(|g| (*g, types.resolve_binding_alias(g).unwrap())).collect()
        )
    }
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

impl TraitConformanceRule {
    // Create a conformance rule that doesn't have generics or requirements.
    pub fn direct(conformance: Rc<TraitConformance>) -> Rc<TraitConformanceRule> {
        Rc::new(TraitConformanceRule {
            generics: Default::default(),
            requirements: Default::default(),
            conformance
        })
    }

    pub fn manual(binding: Rc<TraitBinding>, function_bindings: Vec<(&Rc<FunctionHead>, &Rc<FunctionHead>)>) -> Rc<TraitConformanceRule> {
        Self::direct(
            TraitConformance::new(
                binding,
                HashMap::from_iter(
                    function_bindings.into_iter().map(
                        |(x, y)|
                            (Rc::clone(x), Rc::clone(y)))
                )
            )
        )
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

    pub fn merge(a: &RequirementsFulfillment, b: &RequirementsFulfillment) -> Rc<RequirementsFulfillment> {
        Rc::new(RequirementsFulfillment {
            conformance: a.conformance.clone().into_iter().chain(b.conformance.clone().into_iter()).collect(),
            generic_mapping: a.generic_mapping.clone().into_iter().chain(b.generic_mapping.clone().into_iter()).collect(),
        })
    }
}

impl PartialEq for Trait {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Trait {}

impl Hash for Trait {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Hash for TraitBinding {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.trait_.hash(state);

        for keyval in self.generic_to_type.iter().sorted_by_key(|(id, type_)| *id) {
            keyval.hash(state);
        }
    }
}

impl Debug for Trait {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}<", self.name)?;
        write_keyval(fmt, &self.generics)?;
        write!(fmt, ">")?;

        Ok(())
    }
}

impl Debug for TraitBinding {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}<", self.trait_.name)?;
        write_keyval(fmt, &self.generic_to_type)?;
        write!(fmt, ">")?;

        Ok(())
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

        for (id, type_) in self.generic_mapping.iter().sorted_by_key(|(id, type_)| **id) {
            id.hash(state);
            type_.hash(state);
        }
    }
}
