use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use custom_error::custom_error;
use itertools::{Itertools, zip_eq};
use uuid::Uuid;
use crate::linker::LinkError;
use crate::program::allocation::{ObjectReference, Reference};
use crate::program::functions::{Function, FunctionPointer, FunctionCallType, FunctionInterface, Parameter};
use crate::program::generics::{TypeForest};
use crate::program::types::TypeProto;
use crate::util::fmt::{write_comma_separated_list, write_keyval};
use crate::util::multimap::{extend_multimap, push_into_multimap};

#[derive(Clone)]
pub struct Trait {
    pub id: Uuid,
    pub name: String,

    // You can interpret this like 'inheritance' in other languages
    pub requirements: HashSet<Rc<TraitConformanceRequirement>>,

    pub generics: HashSet<Uuid>,
    pub abstract_functions: HashSet<Rc<Function>>
}

#[derive(Clone)]
pub struct TraitConformanceRequirement {
    pub id: Uuid,
    pub trait_: Rc<Trait>,
    pub binding: HashMap<Uuid, Box<TypeProto>>,
}

#[derive(Clone)]
pub struct TraitConformanceDeclaration {
    pub id: Uuid,
    pub trait_: Rc<Trait>,
    pub binding: HashMap<Uuid, Box<TypeProto>>,
    pub requirements: HashSet<Rc<TraitConformanceRequirement>>,

    pub trait_requirements_conformance: TraitBinding,
    pub abstract_function_resolutions: HashMap<Rc<Function>, Rc<FunctionPointer>>
}

#[derive(Clone, PartialEq, Eq)]
pub struct TraitConformanceScope {
    /// Which declarations are defined in the scope?
    pub declarations: HashMap<Rc<Trait>, Vec<Rc<TraitConformanceDeclaration>>>,
}

#[derive(Clone)]
pub struct TraitBinding {
    pub resolution: HashMap<Rc<TraitConformanceRequirement>, Rc<TraitConformanceDeclaration>>,
}

impl TraitConformanceScope {
    pub fn new() -> TraitConformanceScope {
        TraitConformanceScope {
            declarations: HashMap::new(),
        }
    }

    pub fn merge(lhs: &TraitConformanceScope, rhs: &TraitConformanceScope) -> TraitConformanceScope {
        let mut declarations = lhs.declarations.clone();
        extend_multimap(&mut declarations, &rhs.declarations);

        TraitConformanceScope { declarations }
    }

    pub fn add(&mut self, declaration: &Rc<TraitConformanceDeclaration>) {
        push_into_multimap(&mut self.declarations, &declaration.trait_, Rc::clone(declaration));
    }

    pub fn satisfy_requirements(&self, requirements: &HashSet<Rc<TraitConformanceRequirement>>, mapping: &TypeForest) -> Result<Box<TraitBinding>, LinkError> {
        if requirements.len() == 0 {
            return Ok(Box::new(TraitBinding {
                resolution: HashMap::new(),
            }));
        }

        if requirements.len() > 1 {
            todo!("Multiple requirements are not supported yet")
        }

        let requirement = requirements.iter().next().unwrap();
        let bound_requirement_arguments: HashMap<Uuid, Box<TypeProto>> = requirement.binding.iter()
            .map(|(generic_id, type_)| Ok((*generic_id, mapping.resolve_type(type_)?)))
            .try_collect()?;
        let mut candidates: Vec<Box<TraitBinding>> = vec![];

        for declaration in self.declarations.get(&requirement.trait_).unwrap_or(&vec![]).iter() {
            if !declaration.requirements.is_empty() {
                todo!("Trait conformance declarations with requirements are not supported yet")
            }

            if bound_requirement_arguments != declaration.binding {
                continue
            }

            candidates.push(Box::new(TraitBinding {
                resolution: HashMap::from([(Rc::clone(requirement), Rc::clone(declaration))]),
            }));
        }

        if candidates.len() == 1 {
            return Ok(candidates.into_iter().next().unwrap());
        }

        if candidates.len() > 1 {
            // TODO Due to unbound generics, trait conformance may be coerced later.
            //  However, we don't want to accidentally use another function while this function has ambiguous conformance.
            //  In that case, evaluation should fail when no further generics can be decided.
            panic!("Trait conformance is ambiguous ({}x): {:?}", candidates.len(), requirement);
        }

        Err(LinkError::LinkError { msg: String::from(format!("No compatible declaration for trait conformance requirement: {:?}", requirement)) })
    }
}

impl Trait {
    pub fn new(name: String) -> Rc<Trait> {
        Rc::new(Trait {
            id: Uuid::new_v4(),
            name,
            requirements: Default::default(),
            generics: Default::default(),
            abstract_functions: Default::default(),
        })
    }

    pub fn require(trait_: &Rc<Trait>, binding: HashMap<Uuid, Box<TypeProto>>) -> Rc<TraitConformanceRequirement> {
        Rc::new(TraitConformanceRequirement {
            id: Uuid::new_v4(),
            trait_: Rc::clone(trait_),
            binding
        })
    }

    pub fn assume_granted(trait_: &Rc<Trait>, binding: HashMap<Uuid, Box<TypeProto>>) -> Rc<TraitConformanceDeclaration> {
        let mut replace_map = HashMap::new();
        for generic_id in trait_.generics.iter() {
            replace_map.insert(generic_id.clone(), binding[generic_id].clone());
        }

        let declaration_id = Uuid::new_v4();
        let mut abstract_to_mapped = HashMap::new();

        // Add requirement's implied abstract functions to scope
        for abstract_fun in trait_.abstract_functions.iter() {
            // TODO Re-use existing functions, otherwise we'll have clashes in the scope
            let mapped_pointer = Rc::new(FunctionPointer {
                pointer_id: Uuid::new_v4(),
                call_type: FunctionCallType::Polymorphic {
                    abstract_function: Rc::clone(abstract_fun),
                },
                interface: Rc::new(FunctionInterface {
                    // TODO Mapping variables seems wrong, especially since they are hashable by ID?
                    //  Maybe here, too, there could be a distinction between 'ID of the memory location'
                    //  and 'ID of the pointer to the location'
                    parameters: abstract_fun.interface.parameters.iter().map(|x| {
                        Parameter {
                            external_key: x.external_key.clone(),
                            internal_name: x.internal_name.clone(),
                            target: Rc::new(ObjectReference {
                                id: x.target.id,
                                type_: x.target.type_.replacing_any(&replace_map),
                                mutability: x.target.mutability
                            }),
                        }
                    }).collect(),
                    return_type: abstract_fun.interface.return_type.replacing_any(&replace_map),
                    name: abstract_fun.interface.name.clone(),
                    form: abstract_fun.interface.form.clone(),
                    // Note: abstract functions will never have requirements, because abstract functions are not allowed
                    // any requirements beyond what the trait requires.
                    requirements: HashSet::new(),
                }),
            });

            abstract_to_mapped.insert(Rc::clone(abstract_fun), Rc::clone(&mapped_pointer));
        }

        Rc::new(TraitConformanceDeclaration {
            id: declaration_id,
            trait_: Rc::clone(trait_),
            binding,
            // This declaration can be treated as fulfilled within the scope
            requirements: HashSet::new(),
            trait_requirements_conformance: TraitBinding {
                resolution: trait_.requirements.iter().map(|requirement| {
                    (Rc::clone(requirement), Trait::assume_granted(&requirement.trait_, requirement.binding.iter().map(|(generic_id, type_)| (*generic_id, type_.replacing_any(&replace_map))).collect()))
                }).collect()
            },
            abstract_function_resolutions: abstract_to_mapped
        })
    }
}

impl TraitConformanceDeclaration {
    pub fn make(trait_: &Rc<Trait>, binding: HashMap<Uuid, Box<TypeProto>>, abstract_function_resolutions: Vec<(&Rc<Function>, &Rc<FunctionPointer>)>) -> Rc<TraitConformanceDeclaration> {
        Rc::new(TraitConformanceDeclaration {
            id: Uuid::new_v4(),
            trait_: Rc::clone(trait_),
            binding,
            requirements: HashSet::new(),
            trait_requirements_conformance: TraitBinding::new(),
            abstract_function_resolutions: abstract_function_resolutions.into_iter()
                .map(|(l, r)| (Rc::clone(l), Rc::clone(r)))
                .collect()
        })
    }

    pub fn make_child(trait_: &Rc<Trait>, parent_conformances: Vec<&Rc<TraitConformanceDeclaration>>, abstract_function_resolutions: Vec<(&Rc<Function>, &Rc<FunctionPointer>)>) -> Rc<TraitConformanceDeclaration> {
        Rc::new(TraitConformanceDeclaration {
            id: Uuid::new_v4(),
            trait_: Rc::clone(trait_),
            binding: parent_conformances.iter().next().unwrap().binding.clone(),
            requirements: HashSet::new(),
            trait_requirements_conformance: TraitBinding {
                resolution: parent_conformances.into_iter()
                    .map(|d| (Rc::clone(trait_.requirements.iter().filter(|r| r.trait_ == d.trait_).next().unwrap()), Rc::clone(d)))
                    .collect()
            },
            abstract_function_resolutions: abstract_function_resolutions.into_iter()
                .map(|(l, r)| (Rc::clone(l), Rc::clone(r)))
                .collect()
        })
    }
}

impl TraitConformanceRequirement {
    pub fn with_any_as_generic(&self, seed: &Uuid) -> Rc<TraitConformanceRequirement> {
        Rc::new(TraitConformanceRequirement {
            id: self.id,
            trait_: Rc::clone(&self.trait_),
            binding: self.binding.iter().map(|(generic_id, type_) | (*generic_id, type_.with_any_as_generic(seed))).collect()
        })
    }
}

impl TraitBinding {
    fn new() -> TraitBinding {
        TraitBinding { resolution: HashMap::new() }
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


impl PartialEq for TraitConformanceRequirement {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TraitConformanceRequirement {}

impl Hash for TraitConformanceRequirement {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}


impl PartialEq for TraitConformanceDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TraitConformanceDeclaration {}

impl Hash for TraitConformanceDeclaration {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Debug for TraitConformanceDeclaration {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}<", self.trait_.name)?;
        write_keyval(fmt, &self.binding)?;
        write!(fmt, ">")?;

        Ok(())
    }
}

impl Debug for TraitConformanceRequirement {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}<", self.trait_.name)?;
        write_keyval(fmt, &self.binding)?;
        write!(fmt, ">")?;

        Ok(())
    }
}
