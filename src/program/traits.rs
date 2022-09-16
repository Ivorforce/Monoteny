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
    pub arguments: Vec<Box<Type>>
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

pub type TraitBinding = HashMap<Rc<TraitConformanceRequirement>, Rc<TraitConformanceDeclaration>>;

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
            return Ok(Box::new(HashMap::new()));
        }

        if requirements.len() > 1 {
            todo!("Multiple requirements are not supported yet")
        }

        let requirement = requirements.iter().next().unwrap();
        let bound_requirement_arguments: Vec<Box<Type>> = requirement.arguments.iter()
            .map(|x| mapping.resolve_type(&x.generify(seed)).unwrap())
            .collect();

        let candidates: Vec<Box<TraitBinding>> = self.declarations.get(&requirement.trait_).unwrap_or(&vec![]).iter()
            .flat_map(|d| {
                if !d.requirements.is_empty() {
                    todo!("Trait conformance declarations with requirements are not supported yet")
                }

                if bound_requirement_arguments != d.arguments {
                    return None
                }

                return Some(Box::new(TraitBinding::from([
                    (Rc::clone(requirement), Rc::clone(d))
                ])))
            })
            .collect();

        if candidates.len() == 0 {
            return Err(TraitConformanceError::Error { msg: String::from(format!("No candidates for trait conformance: {}", requirement.trait_.name)) })
        }
        else if candidates.len() > 1 {
            return Err(TraitConformanceError::Error { msg: String::from(format!("Trait conformance is ambiguous: {}", requirement.trait_.name)) })
        }

        return Ok(candidates.into_iter().next().unwrap())
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
