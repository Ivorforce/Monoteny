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

/// The definition of some trait.
#[derive(Clone)]
pub struct Trait {
    pub id: Uuid,
    pub name: String,

    // You can interpret this like 'inheritance' in other languages
    pub requirements: HashSet<Rc<TraitRequirement>>,

    pub generics: HashSet<Uuid>,
    pub abstract_functions: HashSet<Rc<FunctionPointer>>
}

/// Some application of a trait with specific types.
#[derive(Clone, Eq, PartialEq)]
pub struct TraitBinding {
    /// The trait that is bound.
    pub trait_: Rc<Trait>,

    /// A mapping from each of the trait's generics to some type.
    pub generic_to_type: HashMap<Uuid, Box<TypeProto>>,
}

/// A requirement for a trait. Each of the trait's generics are bound to some other type.
/// Note that this requirement is not yet *active* - when somebody references it, all of our
/// generics have to be activated via 'with_any_as_generic'.
#[derive(Clone)]
pub struct TraitRequirement {
    pub id: Uuid,
    pub binding: TraitBinding,
}

/// A pre-checked declaration that some trait is fulfilled with a specific set of arguments.
#[derive(Clone)]
pub struct TraitConformanceDeclaration {
    pub id: Uuid,
    /// The outermost trait binding for which we declare conformance.
    pub binding: TraitBinding,

    /// Requirements to be satisfied before this conformance can be used.
    pub requirements: HashSet<Rc<TraitRequirement>>,

    /// How all of the trait's requirements are satisfied.
    pub trait_resolution: Box<TraitResolution>,
}

/// A collection of bindings for trait requirements.
#[derive(Clone, Eq, PartialEq)]
pub struct TraitResolution {
    /// How requirements are resolved by binding them.
    pub requirement_bindings: HashMap<Rc<TraitRequirement>, TraitBinding>,
    /// How each abstract functions declared in requirements' traits are mapped. schema: abstract_function[resolved_pointer]
    pub function_binding: HashMap<Rc<FunctionPointer>, Rc<FunctionPointer>>
}

/// A collection fo trait conformance declarations. This is used for resolving requirements.
#[derive(Clone, PartialEq, Eq)]
pub struct TraitConformanceScope {
    /// Which declarations are defined in the scope?
    pub declarations: HashMap<Rc<Trait>, Vec<Rc<TraitConformanceDeclaration>>>,
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
        push_into_multimap(&mut self.declarations, &declaration.binding.trait_, Rc::clone(declaration));
    }

    pub fn satisfy_requirements(&self, requirements: &Vec<Box<TraitBinding>>, mapping: &TypeForest) -> Result<Box<TraitResolution>, LinkError> {
        if requirements.len() == 0 {
            return Ok(Box::new(TraitResolution {
                requirement_bindings: Default::default(),
                function_binding: Default::default(),
            }));
        }

        if requirements.len() > 1 {
            todo!("Multiple requirements are not supported yet")
        }

        let requirement = requirements.iter().next().unwrap();
        let resolved_binding = TraitBinding {
            trait_: Rc::clone(&requirement.trait_),
            generic_to_type: requirement.generic_to_type.iter()
                .map(|(generic_id, type_)| Ok((*generic_id, mapping.resolve_type(type_)?)))
                .try_collect()?,
        };
        let mut candidates: Vec<Box<TraitResolution>> = vec![];

        for declaration in self.declarations.get(&requirement.trait_).unwrap_or(&vec![]).iter() {
            if !declaration.requirements.is_empty() {
                todo!("Trait conformance declarations with requirements are not supported yet")
            }

            if resolved_binding != declaration.binding {
                continue
            }

            let mut resolution = declaration.trait_resolution.clone();
            candidates.push(resolution);
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
}

impl TraitConformanceDeclaration {
    /// Make a requirements-free conformance declaration directly from a binding and a function binding.
    pub fn make(binding: TraitBinding, function_binding: Vec<(&Rc<FunctionPointer>, &Rc<FunctionPointer>)>) -> Rc<TraitConformanceDeclaration> {
        let mut trait_resolution = TraitResolution::new();
        trait_resolution.function_binding = function_binding.into_iter()
            .map(|(l, r)| (Rc::clone(l), Rc::clone(r)))
            .collect();

        Rc::new(TraitConformanceDeclaration {
            id: Uuid::new_v4(),
            binding,
            requirements: HashSet::new(),
            trait_resolution,
        })
    }

    /// Make a requirements-free conformance declaration directly from a binding and a function binding, as well as a way to resolve the parent.
    pub fn make_child(trait_: &Rc<Trait>, parent_conformances: Vec<&Rc<TraitConformanceDeclaration>>, function_binding: Vec<(&Rc<FunctionPointer>, &Rc<FunctionPointer>)>) -> Rc<TraitConformanceDeclaration> {
        let mut trait_resolution = TraitResolution::new();
        trait_resolution.function_binding = function_binding.into_iter()
            .map(|(l, r)| (Rc::clone(l), Rc::clone(r)))
            .collect();

        Rc::new(TraitConformanceDeclaration {
            id: Uuid::new_v4(),
            binding: TraitBinding {
                trait_: Rc::clone(trait_),
                generic_to_type: parent_conformances.iter().next().unwrap().binding.generic_to_type.clone(),
            },
            requirements: HashSet::new(),
            trait_resolution,
        })
    }
}

impl TraitRequirement {
    pub fn with_any_as_generic(&self, seed: &Uuid) -> Box<TraitBinding> {
        Box::new(TraitBinding {
            trait_: Rc::clone(&self.binding.trait_),
            generic_to_type: self.binding.generic_to_type.iter().map(|(generic_id, type_) | (*generic_id, type_.with_any_as_generic(seed))).collect()
        })
    }

    pub fn assume_granted(self: &Rc<TraitRequirement>, binding: HashMap<Uuid, Box<TypeProto>>) -> HashMap<Rc<TraitRequirement>, Rc<TraitConformanceDeclaration>> {
        let mut replace_map = HashMap::new();
        for generic_id in self.binding.trait_.generics.iter() {
            replace_map.insert(generic_id.clone(), binding[generic_id].clone());
        }

        let declaration_id = Uuid::new_v4();
        let mut function_binding = HashMap::new();

        // Add requirement's implied abstract functions to scope
        for abstract_fun in self.binding.trait_.abstract_functions.iter() {
            // TODO Re-use existing functions, otherwise we'll have clashes in the scope
            let mapped_pointer = Rc::new(FunctionPointer {
                pointer_id: Uuid::new_v4(),
                call_type: FunctionCallType::Polymorphic { requirement: Rc::clone(self), abstract_function: Rc::clone(abstract_fun) },
                name: abstract_fun.name.clone(),
                form: abstract_fun.form.clone(),
                target: Function::new(Rc::new(FunctionInterface {
                    parameters: abstract_fun.target.interface.parameters.iter().map(|x| {
                        Parameter {
                            external_key: x.external_key.clone(),
                            internal_name: x.internal_name.clone(),
                            // TODO Mapping variables seems wrong, especially since they are hashable by ID?
                            //  Maybe here, too, there could be a distinction between 'ID of the memory location'
                            //  and 'ID of the pointer to the location'
                            target: Rc::new(ObjectReference {
                                id: x.target.id,
                                type_: x.target.type_.replacing_any(&replace_map),
                                mutability: x.target.mutability
                            }),
                        }
                    }).collect(),
                    return_type: abstract_fun.target.interface.return_type.replacing_any(&replace_map),
                    // Note: abstract functions will never have requirements, because abstract functions are not allowed
                    // any requirements beyond what the trait requires.
                    requirements: vec![],
                })),
            });

            function_binding.insert(Rc::clone(abstract_fun), Rc::clone(&mapped_pointer));
        }

        // Assume each of the traits' requirements are granted too
        let mut trait_requirements_implicit_declarations: HashMap<Rc<TraitRequirement>, Rc<TraitConformanceDeclaration>> = Default::default();
        for requirement in self.binding.trait_.requirements.iter() {
            trait_requirements_implicit_declarations.extend(
                requirement.assume_granted(requirement.binding.generic_to_type.iter()
                    .map(|(generic_id, type_)| (*generic_id, type_.replacing_any(&replace_map)))
                    .collect())
            )
        }

        // Our full resolution can be built now
        let mut requirement_bindings: HashMap<Rc<TraitRequirement>, TraitBinding> = [(Rc::clone(self), self.binding.clone())].into_iter().collect();
        for declaration in trait_requirements_implicit_declarations.values() {
            // function_binding.extend(declaration.trait_resolution.function_binding.clone());
            requirement_bindings.extend(declaration.trait_resolution.requirement_bindings.clone());
        }

        // Attach our resolution to the trait's
        trait_requirements_implicit_declarations.into_iter()
            .chain([(Rc::clone(self), Rc::new(TraitConformanceDeclaration {
                id: declaration_id,
                binding: TraitBinding {
                    trait_: Rc::clone(&self.binding.trait_),
                    generic_to_type: binding,
                },
                // This declaration can be treated as fulfilled within the scope
                requirements: HashSet::new(),
                trait_resolution: Box::new(TraitResolution {
                    requirement_bindings,
                    function_binding
                }),
            }))])
            .collect()
    }
}

impl TraitResolution {
    pub fn new() -> Box<TraitResolution> {
        Box::new(TraitResolution { requirement_bindings: Default::default(), function_binding: Default::default() })
    }

    pub fn gather_function_bindings(&self) -> HashMap<Rc<FunctionPointer>, Rc<FunctionPointer>> {
        let mut map = HashMap::new();

        for (requirement, resolution) in self.requirement_bindings.iter() {
            todo!()
            // for (abstract_fun, pointer) in requirement.implicit_declaration.function_binding.iter() {
            //     let function_resolution = &resolution.function_binding[abstract_fun];
            //     map[pointer] = Rc::clone(function_resolution);
            // }
        }

        map
    }

    pub fn gather_type_bindings(&self) -> HashMap<Rc<FunctionPointer>, Rc<FunctionPointer>> {
        todo!()
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


impl PartialEq for TraitRequirement {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TraitRequirement {}

impl Hash for TraitRequirement {
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

impl Hash for TraitBinding {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.trait_.hash(state);

        for keyval in self.generic_to_type.iter().sorted_by_key(|(id, type_)| *id) {
            keyval.hash(state);
        }
    }
}

impl Hash for TraitResolution {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for keyval in self.requirement_bindings.iter().sorted_by_key(|(req, type_)| req.id) {
            keyval.hash(state);
        }

        for keyval in self.function_binding.iter().sorted_by_key(|(ptr, type_)| ptr.pointer_id) {
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
