use std::collections::{HashMap, HashSet};
use std::collections::hash_map::{DefaultHasher, Entry};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use uuid::Uuid;
use crate::linker::LinkError;
use crate::program::functions::{Function, FunctionPointer, FunctionCallType, FunctionInterface, Parameter};
use crate::program::generics::{GenericAlias, TypeForest};
use crate::program::types::{TypeProto, TypeUnit};
use crate::util::fmt::write_keyval;

/// The definition of some trait.
#[derive(Clone)]
pub struct Trait {
    pub id: Uuid,
    pub name: String,

    // Functions required by this trait specifically.
    pub abstract_functions: HashSet<Rc<FunctionPointer>>,
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

/// A sum of knowledge about trait conformance.
/// You can query this to find out if some binding can be cast to some other binding.
/// It caches conformance for subtraits so that lookup is fast.
#[derive(Clone, Eq, PartialEq)]
pub struct TraitGraph {
    /// For each trait, all of its declarations are listed here.
    /// `mapped_function = map[trait][declaration][abstract_function]`
    pub declarations: HashMap<Rc<Trait>, HashMap<Rc<TraitBinding>, HashMap<Rc<FunctionPointer>, Rc<FunctionPointer>>>>,
    /// For each trait, what other traits does it require?
    pub requirements: HashMap<Rc<Trait>, HashSet<Rc<TraitBinding>>>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct RequirementsAssumption {
    pub conformance: HashMap<Rc<TraitBinding>, HashMap<Rc<FunctionPointer>, Rc<FunctionPointer>>>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct RequirementsFulfillment {
    pub conformance: HashMap<Rc<TraitBinding>, HashMap<Rc<FunctionPointer>, Rc<FunctionPointer>>>,
    pub generic_mapping: HashMap<Uuid, Box<TypeProto>>,
}

impl TraitGraph {
    pub fn new() -> TraitGraph {
        TraitGraph {
            declarations: Default::default(),
            requirements: Default::default(),
        }
    }

    pub fn add_graph(&mut self, graph: &TraitGraph) {
        for (trait_, declarations) in graph.declarations.iter() {
            match self.declarations.entry(Rc::clone(trait_)) {
                Entry::Occupied(o) => {
                    o.into_mut().extend(declarations.clone());
                }
                Entry::Vacant(v) => {
                    v.insert(declarations.clone());
                }
            }
        }

        self.requirements.extend(graph.requirements.clone())
    }

    pub fn add_conformance(&mut self, conformance: Rc<TraitBinding>, function_bindings: HashMap<Rc<FunctionPointer>, Rc<FunctionPointer>>) -> Result<(), LinkError> {
        for requirement in self.requirements.get(&conformance.trait_).unwrap_or(&Default::default()) {
            let resolved_requirement = Rc::new(TraitBinding {
                trait_: Rc::clone(&requirement.trait_),
                generic_to_type: requirement.generic_to_type.iter()
                    .map(|(key, value)| (*key, value.replacing_generics(&conformance.generic_to_type)))
                    .collect(),
            });
            if !self.declarations.get(&resolved_requirement.trait_).unwrap_or(&Default::default()).contains_key(&resolved_requirement) {
                return Err(LinkError::LinkError { msg: String::from(format!("{:?} cannot be declared without first declaring its requirement: {:?}", conformance, requirement)) });
            }
        }

        match self.declarations.get_mut(&conformance.trait_) {
            None => {
                // New entry
                self.declarations.insert(Rc::clone(&conformance.trait_), HashMap::from([
                    (Rc::clone(&conformance), function_bindings)
                ]));
            }
            Some(map) => {
                // Expand entry
                map.insert(Rc::clone(&conformance), function_bindings);
            }
        };

        Ok(())
    }

    pub fn add_conformance_manual(&mut self, conformance: Rc<TraitBinding>, function_bindings: Vec<(&Rc<FunctionPointer>, &Rc<FunctionPointer>)>) -> Result<(), LinkError> {
        self.add_conformance(
            conformance,
            HashMap::from_iter(
                function_bindings.into_iter().map(
                    |(x, y)|
                    (Rc::clone(x), Rc::clone(y)))
            )
        )
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

    pub fn satisfy_requirement(&self, requirement: &Rc<TraitBinding>, mapping: &TypeForest) -> Result<HashMap<Rc<FunctionPointer>, Rc<FunctionPointer>>, LinkError> {
        // TODO What if requirement is e.g. Float<Float>? Is Float declared on itself?

        guard!(let Some(relevant_declarations) = self.declarations.get(&requirement.trait_) else {
            return Err(LinkError::LinkError { msg: String::from(format!("No declaration found for trait: {}", &requirement.trait_.name)) });
        });

        // We resolve this binding because it might contain generics.
        let resolved_binding = requirement.try_mapping_types(&|type_| mapping.resolve_type(type_))?;
        if !resolved_binding.collect_generics().is_empty() {
            return Err(LinkError::Ambiguous);
        }

        if let Some(declaration) = relevant_declarations.get(&resolved_binding) {
            // The trait is declared explicitly!
            return Ok(declaration.clone());
        }

        return Err(LinkError::LinkError { msg: String::from(format!("No compatible declaration for trait conformance requirement: {:?}", requirement)) });
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

    pub fn assume_granted<C>(&self, bindings: C) -> Vec<(Rc<TraitBinding>, HashMap<Rc<FunctionPointer>, Rc<FunctionPointer>>)> where C: Iterator<Item=Rc<TraitBinding>> {
        let deep_requirements = self.gather_deep_requirements(bindings);
        let mut resolutions = vec![];

        for trait_binding in deep_requirements.iter() {
            let mut binding_resolution = HashMap::new();

            for abstract_fun in trait_binding.trait_.abstract_functions.iter() {
                let mapped_pointer = Rc::new(FunctionPointer {
                    pointer_id: Uuid::new_v4(),
                    call_type: FunctionCallType::Polymorphic {
                        requirement: Rc::clone(&trait_binding),
                        abstract_function: Rc::clone(abstract_fun)
                    },
                    name: abstract_fun.name.clone(),
                    form: abstract_fun.form.clone(),
                    target: Function::new(Rc::new(FunctionInterface {
                        parameters: abstract_fun.target.interface.parameters.iter().map(|x| {
                            x.mapping_type(&|type_| type_.replacing_generics(&trait_binding.generic_to_type))
                        }).collect(),
                        return_type: abstract_fun.target.interface.return_type.replacing_generics(&trait_binding.generic_to_type),
                        requirements: abstract_fun.target.interface.requirements.iter().map(|req| {
                            req.mapping_types(&|type_| type_.replacing_generics(&trait_binding.generic_to_type))
                        }).collect(),
                    })),
                });

                binding_resolution.insert(
                    Rc::clone(abstract_fun),
                    Rc::clone(&mapped_pointer)
                );
            }

            resolutions.push((Rc::clone(trait_binding), binding_resolution));
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
}

impl RequirementsFulfillment {
    pub fn empty() -> Box<RequirementsFulfillment> {
        Box::new(RequirementsFulfillment {
            conformance: Default::default(),
            generic_mapping: Default::default(),
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
        for (binding, function_mapping) in self.conformance.iter().sorted_by_key(|(binding, mapping)| binding.hash(&mut DefaultHasher::new())) {
            binding.hash(state);
            for keyval in function_mapping.iter().sorted_by_key(|(src, dst)| src.pointer_id) {
                keyval.hash(state)
            }
        }

        for (id, type_) in self.generic_mapping.iter().sorted_by_key(|(id, type_)| *id) {
            id.hash(state);
            type_.hash(state);
        }
    }
}
