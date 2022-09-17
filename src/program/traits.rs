use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use custom_error::custom_error;
use itertools::zip_eq;
use uuid::Uuid;
use crate::program::functions::{FunctionPointer, HumanFunctionInterface, MachineFunctionInterface};
use crate::program::generics::GenericMapping;
use crate::program::types::{Type, Variable};

#[derive(Clone)]
pub struct Trait {
    pub id: Uuid,
    pub name: String,

    // You can interpret this like 'inheritance' in other languages
    pub requirements: HashSet<Rc<TraitConformanceRequirement>>,

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

    pub trait_requirements_conformance: HashMap<Rc<TraitConformanceRequirement>, Rc<TraitConformanceDeclaration>>,
    pub function_implementations: HashMap<Rc<FunctionPointer>, Rc<FunctionPointer>>
}

#[derive(Clone, PartialEq, Eq)]
pub struct TraitConformanceDeclarations {
    declarations: HashMap<Rc<Trait>, Vec<Rc<TraitConformanceDeclaration>>>
}

pub struct TraitBinding {
    pub conformances: HashMap<Rc<TraitConformanceRequirement>, Rc<TraitConformanceDeclaration>>,
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
                conformances: HashMap::new(),
                pointers_resolution: HashMap::new(),
            }));
        }

        if requirements.len() > 1 {
            todo!("Multiple requirements are not supported yet")
        }

        let requirement = requirements.iter().next().unwrap();
        let bound_requirement_arguments: Vec<Box<Type>> = requirement.arguments.iter()
            .map(|x| mapping.resolve_type(&x.with_any_as_generic(seed)).unwrap())
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

        if let Some(conformances) = candidate {
            let mut pointers_resolution = HashMap::new();

            for (requirement, conformance_declaration) in conformances.iter() {
                conformance_declaration.gather_pointer_resolutions(requirement, &mut pointers_resolution);
            }

            return Ok(Box::new(TraitBinding {
                conformances,
                pointers_resolution,
            }));
        }

        panic!("Trait conformance is ambiguous: {}", requirement.trait_.name);
    }
}

impl TraitConformanceRequirement {
    pub fn bind(trait_: &Rc<Trait>, arguments: Vec<Box<Type>>, requirements: &mut HashSet<Rc<TraitConformanceRequirement>>) {
        let mut replace_map = HashMap::new();
        for (param, arg) in zip_eq(trait_.parameters.iter(), arguments.iter()) {
            replace_map.insert(param.clone(), arg.clone());
        }

        let mut functions_pointers = HashMap::new();

        // Add requirement's implied abstract functions to scope
        for abstract_fun in trait_.abstract_functions.iter() {
            // TODO Re-use existing functions, otherwise we'll have clashes in the scope
            let mapped_pointer = Rc::new(FunctionPointer {
                pointer_id: Uuid::new_v4(),
                function_id: abstract_fun.function_id,
                human_interface: Rc::clone(&abstract_fun.human_interface),
                machine_interface: Rc::new(MachineFunctionInterface {
                    // TODO Mapping variables seems wrong, especially since they are hashable by ID?
                    //  Maybe here, too, there could be a distinction between 'ID of the memory location'
                    //  and 'ID of the pointer to the location'
                    parameters: abstract_fun.machine_interface.parameters.iter().map(|x| Rc::new(Variable {
                        id: x.id,
                        type_declaration: x.type_declaration.replacing_any(&replace_map),
                        mutability: x.mutability
                    })).collect(),
                    return_type: abstract_fun.machine_interface.return_type.as_ref().map(|x| x.replacing_any(&replace_map)),
                    // Note: abstract functions will never have injectable pointers, because they're defined
                    //  in the scope of the trait, which already resolves pointers.
                    injectable_pointers: abstract_fun.machine_interface.injectable_pointers.clone(),
                })
            });

            functions_pointers.insert(Rc::clone(abstract_fun), Rc::clone(&mapped_pointer));
        }

        requirements.insert(Rc::new(TraitConformanceRequirement {
            id: Uuid::new_v4(),
            trait_: Rc::clone(trait_),
            arguments,
            functions_pointers
        }));
    }

    pub fn gather_injectable_pointers<'a, I>(requirements: I) -> HashSet<Rc<FunctionPointer>> where I: Iterator<Item=&'a Rc<TraitConformanceRequirement>> {
        let mut injected_pointers = HashSet::new();
        for requirement in requirements {
            requirement.add_injectable_pointers(&mut injected_pointers);
        }
        return injected_pointers
    }

    pub fn add_injectable_pointers(&self, pointers: &mut HashSet<Rc<FunctionPointer>>) {
        pointers.extend(self.functions_pointers.values().map(Rc::clone));
        for requirement in self.trait_.requirements.iter() {
            requirement.add_injectable_pointers(pointers);
        }
    }
}

impl TraitConformanceDeclaration {
    pub fn gather_pointer_resolutions(&self, requirement: &TraitConformanceRequirement, pointers_resolution: &mut HashMap<Rc<FunctionPointer>,Rc<FunctionPointer>>) {
        for (abstract_func, injectable_pointer) in requirement.functions_pointers.iter() {
            let function_implementation = self.function_implementations.get(abstract_func).unwrap();
            pointers_resolution.insert(Rc::clone(injectable_pointer), Rc::clone(function_implementation));
        }

        for (requirement, conformance) in self.trait_requirements_conformance.iter() {
            conformance.gather_pointer_resolutions(requirement, pointers_resolution);
        }
    }

    pub fn create_for_trivial_inheritance(trait_: &Rc<Trait>, parent_conformance: &Rc<TraitConformanceDeclaration>) -> Rc<TraitConformanceDeclaration> {
        Rc::new(TraitConformanceDeclaration {
            id: Uuid::new_v4(),
            trait_: Rc::clone(trait_),
            arguments: parent_conformance.arguments.clone(),
            requirements: HashSet::new(),
            trait_requirements_conformance: zip_eq(trait_.requirements.iter().map(Rc::clone), [parent_conformance].map(Rc::clone)).collect(),
            function_implementations: HashMap::new()
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
