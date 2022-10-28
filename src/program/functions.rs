use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, format, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use itertools::zip_eq;
use uuid::Uuid;
use crate::LinkError;
use crate::program::allocation::{Mutability, ObjectReference, Reference};
use crate::program::traits::{TraitConformanceDeclaration, TraitConformanceRequirement};
use crate::program::types::{TypeProto};

#[derive(Clone, PartialEq, Eq)]
pub enum FunctionForm {
    Global,
    Member,
    Constant,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ParameterKey {
    Positional,
    Name(String),
}

#[derive(Clone, PartialEq, Eq)]
pub enum FunctionPointerTarget {
    Static { implementation_id: Uuid },
    Polymorphic { declaration_id: Uuid, abstract_function: Rc<FunctionPointer> },
}

pub struct FunctionPointer {
    pub pointer_id: Uuid,
    pub target: FunctionPointerTarget,

    pub human_interface: Rc<HumanFunctionInterface>,
    pub machine_interface: Rc<MachineFunctionInterface>,
}

/// Reference to a multiplicity of functions, usually resolved when attempting to call
#[derive(Clone, PartialEq, Eq)]
pub struct FunctionOverload {
    pub pointers: HashSet<Rc<FunctionPointer>>,
    pub name: String,
    pub form: FunctionForm,
}

pub struct HumanFunctionInterface {
    pub name: String,

    pub parameter_names: Vec<(ParameterKey, Rc<ObjectReference>)>,
    pub parameter_names_internal: Vec<String>,

    pub form: FunctionForm,
}

pub struct MachineFunctionInterface {
    pub parameters: HashSet<Rc<ObjectReference>>,
    pub return_type: Box<TypeProto>,
    // Note: This set will almost certainly be larger than actually required, because
    //  it is automatically assembled. To avoid unnecessary arguments,
    //  use an implementation's (if known) hint for which are actually in use.
    pub requirements: HashSet<Rc<TraitConformanceRequirement>>,
}

impl FunctionPointer {
    pub fn make_operator<'a>(alphanumeric_name: &'a str, count: usize, parameter_type: &Box<TypeProto>, return_type: &Box<TypeProto>) -> Rc<FunctionPointer> {
        let parameter_names = (0..count).map(|_| ParameterKey::Positional);
        let parameters: Vec<Rc<ObjectReference>> = (0..count).map(|x| ObjectReference::make_immutable(parameter_type.clone())).collect();

        Rc::new(FunctionPointer {
            pointer_id: Uuid::new_v4(),
            target: FunctionPointerTarget::Static { implementation_id: Uuid::new_v4() },

            human_interface: Rc::new(HumanFunctionInterface {
                name: String::from(alphanumeric_name),
                parameter_names: zip_eq(parameter_names, parameters.iter().map(|x| Rc::clone(x))).collect(),
                parameter_names_internal: vec![],  // TODO Internal names shouldn't need to be specified for builtins?
                form: FunctionForm::Global,
            }),
            machine_interface:Rc::new(MachineFunctionInterface {
                parameters: parameters.into_iter().collect(),
                return_type: return_type.clone(),
                requirements: HashSet::new(),
            })
        })
    }

    pub fn make_global<'a, I>(name: &'a str, parameter_types: I, return_type: Box<TypeProto>) -> Rc<FunctionPointer> where I: Iterator<Item=Box<TypeProto>> {
        let parameters: Vec<Rc<ObjectReference>> = parameter_types
            .map(|x| ObjectReference::make_immutable(x.clone()))
            .collect();
        let parameter_names = (0..parameters.len()).map(|_| ParameterKey::Positional);

        Rc::new(FunctionPointer {
            pointer_id: Uuid::new_v4(),
            target: FunctionPointerTarget::Static { implementation_id: Uuid::new_v4() },

            human_interface: Rc::new(HumanFunctionInterface {
                name: String::from(name),
                parameter_names: zip_eq(parameter_names, parameters.iter().map(|x| Rc::clone(x))).collect(),
                parameter_names_internal: vec![],  // TODO Internal names shouldn't need to be specified for builtins?
                form: FunctionForm::Global,
            }),
            machine_interface:Rc::new(MachineFunctionInterface {
                parameters: parameters.into_iter().collect(),
                return_type: return_type.clone(),
                requirements: HashSet::new(),
            })
        })
    }
}

impl FunctionOverload {
    pub fn from(function: &Rc<FunctionPointer>) -> Rc<FunctionOverload> {
        Rc::new(FunctionOverload {
            pointers: HashSet::from([Rc::clone(function)]),
            name: function.human_interface.name.clone(),
            form: function.human_interface.form.clone(),
        })
    }

    pub fn adding_function(&self, function: &Rc<FunctionPointer>) -> Result<Rc<FunctionOverload>, LinkError> {
        if self.form != function.human_interface.form {
            return Err(LinkError::LinkError { msg: format!("Cannot overload functions and constants.") })
        }

        Ok(Rc::new(FunctionOverload {
            pointers: self.pointers.iter().chain([function]).map(Rc::clone).collect(),
            name: self.name.clone(),
            form: self.form.clone(),
        }))
    }
}

impl PartialEq for FunctionPointer {
    fn eq(&self, other: &Self) -> bool {
        self.pointer_id == other.pointer_id
    }
}

impl Eq for FunctionPointer {}

impl Hash for FunctionPointer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pointer_id.hash(state);
    }
}

impl Debug for HumanFunctionInterface {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        let mut head = 0;

        match self.form {
            FunctionForm::Global => {}
            FunctionForm::Constant => {}
            FunctionForm::Member => {
                write!(fmt, "{:?}.", self.parameter_names.get(head).unwrap().1.type_)?;
                head += 1;
            },
        }

        write!(fmt, "{}(", self.name)?;

        for (name, variable) in self.parameter_names.iter().skip(head) {
            write!(fmt, "{:?}: {:?},", name, variable.type_)?;
        }

        write!(fmt, ")")?;

        Ok(())
    }
}

impl Debug for ParameterKey {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        use ParameterKey::*;
        use crate::program::functions::ParameterKey::Positional;
        match self {
            Name(s) => write!(fmt, "{}", s),
            Positional => write!(fmt, "_"),
        }
    }
}

impl ParameterKey {
    pub fn from_string(s: String) -> ParameterKey {
        if s == "_" { ParameterKey::Positional } else { ParameterKey::Name(s) }
    }
}
