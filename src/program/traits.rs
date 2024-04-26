use std::collections::{HashMap, HashSet};
use std::collections::hash_map::{DefaultHasher, Entry};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use itertools::Itertools;
use uuid::Uuid;

use crate::error::{RResult, RuntimeError};
use crate::resolver::ambiguous::AmbiguityResult;
use crate::program::function_object::FunctionRepresentation;
use crate::program::functions::{FunctionHead, FunctionInterface, FunctionType};
use crate::program::generics::{GenericAlias, TypeForest};
use crate::program::types::{TypeProto, TypeUnit};
use crate::util::fmt::{write_separated_display, write_keyval};
use crate::util::hash;

/// The definition of some trait.
#[derive(Clone)]
pub struct Trait {
    pub id: Uuid,
    pub name: String,

    // Generics declared for this trait, by name (via its declaration).
    // Used in abstract functions and requirements (collect_generics on those would yield the same GenericAliases).
    pub generics: HashMap<String, Rc<Trait>>,

    // To conform to this trait, these other conformances are required.
    pub requirements: HashSet<Rc<TraitBinding>>,

    // Functions required by this trait specifically (not its requirements).
    // The head of each function to its pointer (how it is defined).
    pub abstract_functions: HashMap<Rc<FunctionHead>, FunctionRepresentation>,
    pub field_hints: Vec<FieldHint>,
}

/// For traits, information about certain fields that have been declared.
///  This is useful mostly if somebody wants to instantiate the trait without sub-traiting it.
#[derive(Clone)]
pub struct FieldHint {
    pub name: String,
    pub type_: Rc<TypeProto>,
    pub setter: Option<Rc<FunctionHead>>,
    pub getter: Option<Rc<FunctionHead>>,
}

/// Some application of a trait with specific types.
#[derive(Clone, Eq, PartialEq)]
pub struct TraitBinding {
    /// The trait that is bound.
    pub trait_: Rc<Trait>,

    /// A mapping from each of the trait's generics to some type.
    pub generic_to_type: HashMap<Rc<Trait>, Rc<TypeProto>>,
}

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

/// Declares conformance of a trait to another trait.
///  For example, a rule may declare:
///     Generic #A
///     Requirement Float32<self: #A>
///     Conformance Number<self: #A>
/// The conformance object then holds what functions fulfill the requirements of Number<self>.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TraitConformanceRule {
    /// Generics declared for this conformance, by name (via its declaration).
    /// Used in requirements and the conformance itself (collect_generics on those would yield the same GenericAliases).
    pub generics: HashMap<String, Rc<Trait>>,

    /// To use this conformance, these other conformances are required.
    pub requirements: HashSet<Rc<TraitBinding>>,

    /// The conformance (w.r.t. generics) defined by this rule.
    pub conformance: Rc<TraitConformance>,
}

/// A sum of knowledge about trait conformance.
/// You can query this to find out if some binding can be cast to some other binding.
/// It caches conformance for subtraits so that lookup is fast.
#[derive(Clone, Eq, PartialEq)]
pub struct TraitGraph {
    /// All known conformances.
    /// For each conformance, we also know its tail, aka how it was achieved.
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
    pub generic_mapping: HashMap<Rc<Trait>, Rc<TypeProto>>,
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

    // TODO This should not return an ambiguity result. The caller should make sure to resolve types, and we should just do our jobs.
    //  Any layers deeper cannot yield ::Ambiguous anyway, if all bindings are properly filled.
    pub fn satisfy_requirement(&mut self, requirement: &Rc<TraitBinding>, mapping: &TypeForest) -> RResult<AmbiguityResult<Rc<TraitConformanceWithTail>>> {
        // TODO What if requirement is e.g. Float<Float>? Is Float declared on itself?

        // We resolve this binding because it might contain generics.
        let resolved_binding = requirement.try_mapping_types(&|type_| mapping.resolve_type(type_))?;
        if !resolved_binding.collect_generics().is_empty() {
            return Ok(AmbiguityResult::Ambiguous);
        }

        if let Some(state) = self.conformance_cache.get(&resolved_binding) {
            // In cache
            return match state {
                None => Err(
                    RuntimeError::error(format!("No compatible declaration for trait conformance requirement: {:?}", resolved_binding).as_str()).to_array()
                ),
                Some(declaration) => Ok(AmbiguityResult::Ok(declaration.clone())),
            };
        }

        let Some(relevant_declarations) = self.conformance_rules.get(&resolved_binding.trait_) else {
            return Err(
                RuntimeError::error(format!("No declarations found for trait: {:?}", resolved_binding.trait_).as_str()).to_array()
            );
        };

        let mut compatible_conformances = vec![];
        let mut bind_errors = vec![];
        let mut requirements_errors = vec![];

        // Recalculate
        // TODO clone is a bit much, but we need it to be memory safe
        let cloned_declarations: Vec<Rc<TraitConformanceRule>> = relevant_declarations.clone();
        'rule: for rule in cloned_declarations.iter() {
            // For a rule to be compatible, its binding must be compatible with the binding from the arguments.
            //  So we create a new TypeForest where we can bind them together.
            let mut rule_mapping = mapping.clone();

            // A rule may also use generics. Those need to be rebindable, and we need to be able to figure out
            //  how they've been bound in the end. To do that, we'll just map them to generics and query those
            //  generics later on.
            let rule_generics_map = rule.generics.values()
                .map(|generic| (Rc::clone(generic), TypeProto::unit(TypeUnit::Generic(Uuid::new_v4()))))
                .collect();

            // Bind together the rule and argument.
            for (key, type_) in rule.conformance.binding.generic_to_type.iter() {
                let tmp_id = Uuid::new_v4();
                rule_mapping.bind(tmp_id, &type_.replacing_structs(&rule_generics_map)).unwrap();

                let resolved_type = &resolved_binding.generic_to_type[key];
                if let Err(err) = rule_mapping.bind(tmp_id, resolved_type) {
                    bind_errors.push(
                        RuntimeError::error(format!("{:?} failed type check.", type_).as_str())
                            .with_notes(err.into_iter())
                    );
                    // Binding failed; this rule is not compatible.
                    continue 'rule;
                }
            }

            match self.test_requirements(&rule.requirements, &rule_generics_map, &rule_mapping) {
                // Can't use this candidate: While it is compatible, its requirements are not fulfilled.
                Err(err) => requirements_errors.push(
                    RuntimeError::error("Failed requirements test.")
                        .with_notes(err.into_iter())
                ),
                Ok(AmbiguityResult::Ambiguous) => {
                    // This shouldn't happen because Ambiguous is only thrown when any requirements have
                    //  unbound generics. We resolved those generics using the binding from earlier.
                    panic!("Got an ambiguity in deep trait resolving.")
                }
                Ok(AmbiguityResult::Ok(fulfilled_requirements)) => {
                    // We can use this candidate! Let's clean it up for use.

                    // Find out how the rule's generics were mapped. This will be our tail.
                    let generic_mapping = rule_generics_map.into_iter().map(|(interface_generic, tmp_generic)| {
                        (interface_generic, rule_mapping.resolve_type(&tmp_generic).unwrap())
                    }).collect();

                    let resolved_conformance = TraitConformance::new(
                        resolved_binding.clone(),
                        // TODO Do we need to map the functions?
                        rule.conformance.function_mapping.clone(),
                    );
                    compatible_conformances.push(
                        Rc::new(TraitConformanceWithTail {
                            tail: Rc::new(RequirementsFulfillment {
                                conformance: fulfilled_requirements,
                                generic_mapping,
                            }),
                            conformance: resolved_conformance,
                        })
                    );
                }
            }
        }

        match compatible_conformances.as_slice() {
            [] => {
                let error = RuntimeError::error(format!("No compatible declaration for trait conformance requirement: {:?}", resolved_binding).as_str());

                self.conformance_cache.insert(Rc::clone(&resolved_binding), None);
                if !requirements_errors.is_empty() {
                    Err(
                        error.with_note(
                            RuntimeError::info(format!("{} rule(s) match types, but their requirements were not satisfied.", requirements_errors.len()).as_str())
                                .with_notes(requirements_errors.into_iter())
                        ).to_array()
                    )
                }
                else {
                    Err(
                        error.with_note(
                            RuntimeError::info(format!("{} rule(s) failed the type check.", bind_errors.len()).as_str())
                                .with_notes(bind_errors.into_iter())
                        ).to_array()
                    )
                }
            }
            [declaration] => {
                self.conformance_cache.insert(resolved_binding, Some(Rc::clone(declaration)));
                Ok(AmbiguityResult::Ok(Rc::clone(declaration)))
            }
            _ => {
                Err(
                    RuntimeError::error(format!("Conflicting declarations for trait conformance requirement: {:?}", resolved_binding).as_str()).with_note(
                        RuntimeError::info(format!("{} matching rule(s).", cloned_declarations.len()).as_str())
                            .with_notes(cloned_declarations.iter().map(|c| RuntimeError::info(format!("{:?}", c).as_str())))
                    ).to_array()
                )
            }
        }
    }

    pub fn test_requirements(&mut self, requirements: &HashSet<Rc<TraitBinding>>, generics_map: &HashMap<Rc<Trait>, Rc<TypeProto>>, mapping: &TypeForest) -> RResult<AmbiguityResult<HashMap<Rc<TraitBinding>, Rc<TraitConformanceWithTail>>>> {
        let mut conformance = HashMap::new();

        for requirement in self.gather_deep_requirements(requirements.iter().cloned()) {
            let mapped_requirement = requirement.mapping_types(&|t| t.replacing_structs(generics_map));

            match self.satisfy_requirement(&mapped_requirement, &mapping)? {
                AmbiguityResult::Ok(trait_conformance) => {
                    conformance.insert(requirement.clone(), trait_conformance);
                }
                AmbiguityResult::Ambiguous => {
                    return Ok(AmbiguityResult::Ambiguous)
                }
            }
        }

        Ok(AmbiguityResult::Ok(conformance))
    }

    /// This function takes in some 'explicit' requirements,
    ///  and returns a vector of all requirements these imply, explicit or implicit.
    pub fn gather_deep_requirements<C>(&self, bindings: C) -> Vec<Rc<TraitBinding>> where C: Iterator<Item=Rc<TraitBinding>> {
        let mut all = HashSet::new();
        let mut ordered = vec![];
        let mut rest = bindings.collect_vec();
        while let Some(binding) = rest.pop() {
            if all.insert(Rc::clone(&binding)) {
                ordered.push(Rc::clone(&binding));
                rest.extend(
                    binding.trait_.requirements.iter()
                        .map(|x| x.mapping_types(&|type_| type_.replacing_structs(&binding.generic_to_type))))
            }
        }
        ordered.reverse();
        ordered
    }

    /// This function takes in a bunch of 'explicit' requirements, and creates a conformance
    ///  for all requirements implied by those implicit requirements.
    /// To do this, it invents function stubs that will later have to be replaced by the actual
    ///  functions that the caller provides.
    pub fn assume_granted(&self, bindings: impl Iterator<Item=Rc<TraitBinding>>) -> Vec<Rc<TraitConformance>> {
        let deep_requirements = self.gather_deep_requirements(bindings);
        let mut resolutions = vec![];

        for requirement in deep_requirements.iter() {
            let mut binding_resolution = HashMap::new();

            for abstract_fun in requirement.trait_.abstract_functions.keys() {
                let mapped_head = FunctionHead::new(
                    Rc::new(FunctionInterface {
                        parameters: abstract_fun.interface.parameters.iter().map(|x| {
                            x.mapping_type(&|type_| type_.replacing_structs(&requirement.generic_to_type))
                        }).collect(),
                        return_type: abstract_fun.interface.return_type.replacing_structs(&requirement.generic_to_type),
                        requirements: abstract_fun.interface.requirements.iter().map(|req| {
                            req.mapping_types(&|type_| type_.replacing_structs(&requirement.generic_to_type))
                        }).collect(),
                        // the function's own generics aren't mapped; we're only binding those from the trait itself.
                        generics: abstract_fun.interface.generics.clone(),
                    }),
                    FunctionType::Polymorphic {
                        assumed_requirement: Rc::clone(&requirement),
                        abstract_function: Rc::clone(abstract_fun)
                    }
                );
                binding_resolution.insert(
                    Rc::clone(&abstract_fun),
                    mapped_head
                );
            }

            resolutions.push(
                TraitConformance::new(Rc::clone(requirement), binding_resolution)
            );
        }

        resolutions
    }
}

impl Trait {
    pub fn new_flat(name: &str) -> Trait {
        Trait {
            id: Uuid::new_v4(),
            name: name.to_string(),
            generics: Default::default(),
            requirements: Default::default(),
            abstract_functions: Default::default(),
            field_hints: Default::default(),
        }
    }

    pub fn new_with_self(name: &str) -> Trait {
        Trait {
            id: Uuid::new_v4(),
            name: name.to_string(),
            generics: HashMap::from([("Self".to_string(), Rc::new(Trait::new_flat("Self")))]),
            requirements: Default::default(),
            abstract_functions: Default::default(),
            field_hints: Default::default(),
        }
    }

    pub fn create_generic_type(self: &Trait, generic_name: &str) -> Rc<TypeProto> {
        TypeProto::unit_struct(&self.generics[generic_name])
    }

    pub fn create_generic_binding(self: &Rc<Trait>, generic_to_type: Vec<(&str, Rc<TypeProto>)>) -> Rc<TraitBinding> {
        Rc::new(TraitBinding {
            trait_: Rc::clone(self),
            generic_to_type: HashMap::from_iter(
                generic_to_type.into_iter()
                    .map(|(generic_name, type_)| (Rc::clone(&self.generics[generic_name]), type_))
            ),
        })
    }

    pub fn insert_function(&mut self, function: Rc<FunctionHead>, representation: FunctionRepresentation) {
        self.abstract_functions.insert(function, representation);
    }

    pub fn add_simple_parent_requirement(&mut self, parent_trait: &Rc<Trait>) {
        self.requirements.insert(
            parent_trait.create_generic_binding(vec![("Self", self.create_generic_type("Self"))])
        );
    }
}

impl TraitBinding {
    pub fn mapping_types(&self, map: &dyn Fn(&Rc<TypeProto>) -> Rc<TypeProto>) -> Rc<TraitBinding> {
        Rc::new(TraitBinding {
            trait_: Rc::clone(&self.trait_),
            generic_to_type: self.generic_to_type.iter().map(|(generic, type_) | (Rc::clone(generic), map(type_))).collect()
        })
    }

    pub fn try_mapping_types<B>(&self, map: &dyn Fn(&Rc<TypeProto>) -> Result<Rc<TypeProto>, B>) -> Result<Rc<TraitBinding>, B> {
        Ok(Rc::new(TraitBinding {
            trait_: Rc::clone(&self.trait_),
            generic_to_type: self.generic_to_type.iter().map(|(generic, type_) | Ok((Rc::clone(generic), map(type_)?))).try_collect()?
        }))
    }

    pub fn collect_generics(&self) -> HashSet<GenericAlias> {
        TypeProto::collect_generics(self.generic_to_type.values())
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

        for keyval in self.generic_to_type.iter().sorted_by_key(|(trait_, type_)| trait_.id) {
            keyval.hash(state);
        }
    }
}

impl Debug for Trait {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}<{}>", self.name, self.id)?;
        if !self.generics.is_empty() {
            write!(fmt, "<")?;
            write_separated_display(fmt, ", ", self.generics.keys())?;
            write!(fmt, ">")?;
        }
        Ok(())
    }
}

impl Debug for TraitBinding {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", self.trait_.name)?;
        if !self.generic_to_type.is_empty() {
            write!(fmt, "<")?;
            write_keyval(fmt, &self.generic_to_type)?;
            write!(fmt, ">")?;
        }
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

        for (id, type_) in self.generic_mapping.iter().sorted_by_key(|(trait_, type_)| trait_.id) {
            id.hash(state);
            type_.hash(state);
        }
    }
}
