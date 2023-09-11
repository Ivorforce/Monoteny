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

    // Functions required by this trait specifically.
    pub abstract_functions: HashMap<Rc<FunctionHead>, Rc<FunctionPointer>>,
    // Generics declared by this trait, by name (via its declaration).
    // May be used in abstract functions and requirements.
    pub generics: HashMap<String, GenericAlias>,
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
    pub binding: Rc<TraitBinding>,
    pub function_mapping: HashMap<Rc<FunctionHead>, Rc<FunctionHead>>,
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
    pub conformance: HashMap<Rc<TraitBinding>, Option<(Box<RequirementsFulfillment>, Rc<TraitConformance>)>>,
    /// For each trait, what other traits does it require?
    /// This causes cascading requirements on functions etc.
    pub requirements: HashMap<Rc<Trait>, HashSet<Rc<TraitBinding>>>,
    /// A list of conformance declarations that allow for dynamic conformance.
    /// All these use generics in the conformance, which are provided by the requirements.
    /// To use the conformance, these generics should be replaced by the matching bindings.
    pub conformance_rules: HashMap<Rc<Trait>, Vec<(HashSet<Rc<TraitBinding>>, Rc<TraitConformance>)>>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct RequirementsAssumption {
    pub conformance: HashMap<Rc<TraitBinding>, Rc<TraitConformance>>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct RequirementsFulfillment {
    pub conformance: HashMap<Rc<TraitBinding>, (Box<RequirementsFulfillment>, Rc<TraitConformance>)>,
    pub generic_mapping: HashMap<Uuid, Box<TypeProto>>,
}

impl TraitGraph {
    pub fn new() -> TraitGraph {
        TraitGraph {
            conformance: Default::default(),
            requirements: Default::default(),
            conformance_rules: Default::default(),
        }
    }

    pub fn add_graph(&mut self, graph: &TraitGraph) {
        // TODO Check for conflicting conformance
        self.conformance.extend(graph.conformance.clone());
        self.requirements.extend(graph.requirements.clone())
    }

    pub fn add_conformance(&mut self, conformance: Rc<TraitConformance>) -> Result<(), LinkError> {
        // Check if all the requirements are satisfied
        for requirement in self.requirements.get(&conformance.binding.trait_).unwrap_or(&Default::default()) {
            let resolved_requirement = Rc::new(TraitBinding {
                trait_: Rc::clone(&requirement.trait_),
                generic_to_type: requirement.generic_to_type.iter()
                    .map(|(key, value)| (*key, value.replacing_generics(&conformance.binding.generic_to_type)))
                    .collect(),
            });
            if !self.conformance.contains_key(&resolved_requirement) {
                return Err(LinkError::LinkError { msg: String::from(format!("{:?} cannot be declared without first declaring its requirement: {:?}", conformance.binding, requirement)) });
            }
        }

        self.conformance.insert(Rc::clone(&conformance.binding), Some((RequirementsFulfillment::empty(), conformance)));

        Ok(())
    }

    pub fn add_conformance_manual(&mut self, binding: Rc<TraitBinding>, function_bindings: Vec<(&Rc<FunctionHead>, &Rc<FunctionHead>)>) -> Result<(), LinkError> {
        self.add_conformance(
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

    pub fn add_conformance_rule(&mut self, requirements: HashSet<Rc<TraitBinding>>, conformance: Rc<TraitConformance>) {
        match self.conformance_rules.entry(Rc::clone(&conformance.binding.trait_)) {
            Entry::Occupied(e) => {
                e.into_mut().push((requirements, conformance));
            }
            Entry::Vacant(e) => {
                e.insert(vec![(requirements, conformance)]);
            }
        };
    }

    pub fn add_requirement(&mut self, trait_: Rc<Trait>, requirement: Rc<TraitBinding>) {
        match self.requirements.entry(trait_) {
            Entry::Occupied(o) => {
                o.into_mut().insert(requirement);
            }
            Entry::Vacant(v) => {
                v.insert(HashSet::from([requirement]));
            }
        };
    }

    pub fn add_simple_parent_requirement(&mut self, sub_trait: &Rc<Trait>, parent_trait: &Rc<Trait>) {
        self.add_requirement(
            Rc::clone(sub_trait),
            parent_trait.create_generic_binding(vec![(&"self".into(), sub_trait.create_generic_type(&"self".into()))])
        );
    }

    pub fn satisfy_requirement(&mut self, requirement: &Rc<TraitBinding>, mapping: &TypeForest) -> Result<(Box<RequirementsFulfillment>, Rc<TraitConformance>), LinkError> {
        // TODO What if requirement is e.g. Float<Float>? Is Float declared on itself?

        // We resolve this binding because it might contain generics.
        let resolved_binding = requirement.try_mapping_types(&|type_| mapping.resolve_type(type_))?;
        if !resolved_binding.collect_generics().is_empty() {
            return Err(LinkError::Ambiguous);
        }

        if let Some(state) = self.conformance.get(&resolved_binding) {
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
        for (requirements, offered_conformance) in relevant_declarations.clone().iter() {
            guard!(let Some(generics_map) = TraitBinding::merge(&offered_conformance.binding, &resolved_binding) else {
                continue;
            });
            let resolved_requirements: HashSet<Rc<TraitBinding>> = requirements.iter().map(|x| x.mapping_types(&|type_| type_.replacing_generics(&generics_map))).collect();

            if let Ok(fulfilled_requirements) = self.test_requirements(&resolved_requirements, mapping) {
                let resolved_conformance = TraitConformance::new(
                    resolved_binding.clone(),
                    // TODO Do we need to map the functions?
                    offered_conformance.function_mapping.clone(),
                );
                // TODO There may be more than one conflicting solution
                let pair = (fulfilled_requirements, resolved_conformance);
                self.conformance.insert(resolved_binding, Some(pair.clone()));
                return Ok(pair.clone());
            }
        }

        self.conformance.insert(Rc::clone(&resolved_binding), None);
        Err(LinkError::LinkError { msg: String::from(format!("No compatible declaration for trait conformance requirement: {:?}", resolved_binding)) })
    }

    pub fn test_requirements(&mut self, requirements: &HashSet<Rc<TraitBinding>>, mapping: &TypeForest) -> Result<Box<RequirementsFulfillment>, LinkError> {
        let mut fulfillment = RequirementsFulfillment::empty();

        for r in requirements.iter() {
            let (req_tail, req_fulfillment) = self.satisfy_requirement(r, mapping)?;
            fulfillment.generic_mapping.extend(req_fulfillment.binding.generic_to_type.clone());
            fulfillment.conformance.insert(Rc::clone(r), (req_tail, req_fulfillment));
        }

        return Ok(fulfillment);
    }

    pub fn gather_deep_requirements<C>(&self, bindings: C) -> Vec<Rc<TraitBinding>> where C: Iterator<Item=Rc<TraitBinding>> {
        let mut all = HashSet::new();
        let mut ordered = vec![];
        let mut rest = bindings.collect_vec();
        while let Some(binding) = rest.pop() {
            if all.insert(Rc::clone(&binding)) {
                ordered.push(Rc::clone(&binding));
                rest.extend(
                    self.requirements.get(&binding.trait_)
                        .unwrap_or(&Default::default()).iter()
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
    pub fn new(name: String) -> Trait {
        Trait {
            id: Uuid::new_v4(),
            name,
            abstract_functions: Default::default(),
            generics: HashMap::from([("self".into(), Uuid::new_v4())]),
        }
    }

    pub fn create_generic_type(self: &Trait, generic_name: &String) -> Box<TypeProto> {
        TypeProto::unit(TypeUnit::Generic(self.generics[generic_name]))
    }

    pub fn create_generic_binding(self: &Rc<Trait>, generic_to_type: Vec<(&String, Box<TypeProto>)>) -> Rc<TraitBinding> {
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

impl RequirementsFulfillment {
    pub fn empty() -> Box<RequirementsFulfillment> {
        Box::new(RequirementsFulfillment {
            conformance: Default::default(),
            generic_mapping: Default::default(),
        })
    }

    pub fn is_empty(&self) -> bool {
        self.conformance.is_empty() && self.generic_mapping.is_empty()
    }

    pub fn merge(a: &RequirementsFulfillment, b: &RequirementsFulfillment) -> Box<RequirementsFulfillment> {
        Box::new(RequirementsFulfillment {
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

impl Hash for RequirementsFulfillment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (binding, (conformance_tail, conformance)) in self.conformance.iter().sorted_by_key(|(binding, mapping)| hash::one(binding, DefaultHasher::new())) {
            binding.hash(state);
            conformance_tail.hash(state);
            for keyval in conformance.function_mapping.iter().sorted_by_key(|(src, dst)| src.function_id) {
                keyval.hash(state)
            }
        }

        for (id, type_) in self.generic_mapping.iter().sorted_by_key(|(id, type_)| id.clone()) {
            id.hash(state);
            type_.hash(state);
        }
    }
}
