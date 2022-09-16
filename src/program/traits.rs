use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use custom_error::custom_error;
use uuid::Uuid;
use crate::program::functions::{FunctionPointer, HumanFunctionInterface};
use crate::program::generics::GenericMapping;
use crate::program::types::{Type};

#[derive(Clone)]
pub struct Trait {
    pub id: Uuid,
    pub name: String,

    pub parameters: Vec<Uuid>,

    pub abstract_functions: HashSet<Rc<FunctionPointer>>
}

custom_error!{pub TraitConformanceError
    Error{msg: String} = "Trait Conformance Error: {msg}",
}

#[derive(Clone)]
pub struct TraitConformanceRequirement {
    pub id: Uuid,
    pub trait_: Rc<Trait>,
    pub arguments: Vec<Box<Type>>,

    pub functions_pointers: HashMap<Rc<FunctionPointer>, Rc<FunctionPointer>>,
}

#[derive(Clone)]
pub struct TraitConformanceDeclaration {
    pub id: Uuid,
    pub trait_: Rc<Trait>,
    pub arguments: Vec<Box<Type>>,
    pub requirements: HashSet<Rc<TraitConformanceRequirement>>,

    pub function_implementations: HashMap<Rc<FunctionPointer>, Rc<FunctionPointer>>
}

#[derive(Clone, PartialEq, Eq)]
pub struct TraitConformanceDeclarations {
    declarations: HashMap<Rc<Trait>, Vec<Rc<TraitConformanceDeclaration>>>
}

pub struct TraitBinding {
    pub conformance: HashMap<Rc<TraitConformanceRequirement>, Rc<TraitConformanceDeclaration>>,
    pub pointers_resolution: HashMap<Rc<FunctionPointer>, Rc<FunctionPointer>>,
}


impl TraitConformanceDeclarations {
    pub fn new() -> TraitConformanceDeclarations {
        TraitConformanceDeclarations {
            declarations: HashMap::new()
        }
    }

    pub fn merge(lhs: &TraitConformanceDeclarations, rhs: &TraitConformanceDeclarations) -> TraitConformanceDeclarations {
        let mut copy = lhs.declarations.clone();
        for (trait_, declarations) in rhs.declarations.iter() {
            if let Some(existing) = copy.get_mut(trait_.as_ref()) {
                existing.extend(declarations.clone());
            }
            else {
                copy.insert(trait_.clone(), declarations.clone());
            }
        }

        TraitConformanceDeclarations { declarations: copy }
    }

    pub fn add(&mut self, declaration: Rc<TraitConformanceDeclaration>) {
        if let Some(existing) = self.declarations.get_mut(declaration.trait_.as_ref()) {
            existing.push(declaration);
        }
        else {
            self.declarations.insert(declaration.trait_.clone(), vec![declaration]);
        }
    }

    pub fn satisfy_requirements(&self, requirements: &HashSet<Rc<TraitConformanceRequirement>>, seed: &Uuid, mapping: &GenericMapping) -> Result<Box<TraitBinding>, TraitConformanceError> {
        if requirements.len() == 0 {
            return Ok(Box::new(TraitBinding {
                conformance: HashMap::new(),
                pointers_resolution: HashMap::new(),
            }));
        }

        if requirements.len() > 1 {
            todo!("Multiple requirements are not supported yet")
        }

        let requirement = requirements.iter().next().unwrap();
        let bound_requirement_arguments: Vec<Box<Type>> = requirement.arguments.iter()
            .map(|x| mapping.resolve_type(&x.generify(seed)).unwrap())
            .collect();

        let mut candidate: Option<HashMap<Rc<TraitConformanceRequirement>, Rc<TraitConformanceDeclaration>>> = None;

        for declaration in self.declarations.get(&requirement.trait_).unwrap_or(&vec![]).iter() {
            if !declaration.requirements.is_empty() {
                todo!("Trait conformance declarations with requirements are not supported yet")
            }

            if bound_requirement_arguments != declaration.arguments {
                return continue
            }

            if candidate.is_some() {
                return Err(TraitConformanceError::Error { msg: String::from(format!("No candidates for trait conformance: {}", requirement.trait_.name)) })
            }

            candidate = Some(HashMap::from([
                (Rc::clone(requirement), Rc::clone(declaration))
            ]));
        }

        if let Some(conformance) = candidate {
            let mut pointers_resolution = HashMap::new();

            for (requirement, declaration) in conformance.iter() {
                for (abstract_func, injectable_pointer) in requirement.functions_pointers.iter() {
                    let function_implementation = declaration.function_implementations.get(abstract_func).unwrap();
                    pointers_resolution.insert(Rc::clone(injectable_pointer), Rc::clone(function_implementation));
                }
            }

            return Ok(Box::new(TraitBinding {
                conformance,
                pointers_resolution,
            }));
        }

        panic!("Trait conformance is ambiguous: {}", requirement.trait_.name);
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
