use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use itertools::zip_eq;
use uuid::Uuid;
use crate::program::allocation::{Mutability, Variable};
use crate::program::traits::{TraitConformanceDeclaration, TraitConformanceRequirement};
use crate::program::types::TypeProto;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FunctionForm {
    Global,
    Member,
    Operator,
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

pub struct HumanFunctionInterface {
    pub name: String,
    pub alphanumeric_name: String,

    pub parameter_names: Vec<(ParameterKey, Rc<Variable>)>,
    pub parameter_names_internal: Vec<String>,

    pub form: FunctionForm,
}

pub struct MachineFunctionInterface {
    pub parameters: HashSet<Rc<Variable>>,
    pub return_type: Box<TypeProto>,
    // Note: This set will almost certainly be larger than actually required, because
    //  it is automatically assembled. To avoid unnecessary arguments,
    //  use an implementation's (if known) hint for which are actually in use.
    pub requirements: HashSet<Rc<TraitConformanceRequirement>>,
}

impl FunctionPointer {
    pub fn make_operator<'a>(name: &'a str, alphanumeric_name: &'a str, count: usize, parameter_type: &Box<TypeProto>, return_type: &Box<TypeProto>) -> Rc<FunctionPointer> {
        let parameter_names = (0..count).map(|_| ParameterKey::Positional);
        let parameters: Vec<Rc<Variable>> = (0..count).map(|x| Rc::new(Variable {
            id: Uuid::new_v4(),
            type_declaration: parameter_type.clone(),
            mutability: Mutability::Immutable
        })).collect();

        Rc::new(FunctionPointer {
            pointer_id: Uuid::new_v4(),
            target: FunctionPointerTarget::Static { implementation_id: Uuid::new_v4() },

            human_interface: Rc::new(HumanFunctionInterface {
                name: String::from(name),
                alphanumeric_name: String::from(alphanumeric_name),
                parameter_names: zip_eq(parameter_names, parameters.iter().map(|x| Rc::clone(x))).collect(),
                parameter_names_internal: vec![],  // TODO Internal names shouldn't need to be specified for builtins?
                form: FunctionForm::Operator,
            }),
            machine_interface:Rc::new(MachineFunctionInterface {
                parameters: parameters.into_iter().collect(),
                return_type: return_type.clone(),
                requirements: HashSet::new(),
            })
        })
    }

    pub fn make_global<'a, I>(name: &'a str, alphanumeric_name: &'a str, parameter_types: I, return_type: Box<TypeProto>) -> Rc<FunctionPointer> where I: Iterator<Item=Box<TypeProto>> {
        let parameters: Vec<Rc<Variable>> = parameter_types
            .map(|x| Variable::make_immutable(x.clone()))
            .collect();
        let parameter_names = (0..parameters.len()).map(|_| ParameterKey::Positional);

        Rc::new(FunctionPointer {
            pointer_id: Uuid::new_v4(),
            target: FunctionPointerTarget::Static { implementation_id: Uuid::new_v4() },

            human_interface: Rc::new(HumanFunctionInterface {
                name: String::from(name),
                alphanumeric_name: String::from(alphanumeric_name),
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
            FunctionForm::Member => {
                write!(fmt, "{:?}.", self.parameter_names.get(head).unwrap().1.type_declaration)?;
                head += 1;
            },
            // TODO Unary operators?
            FunctionForm::Operator => {}
        }

        write!(fmt, "{}(", self.name)?;

        for (name, variable) in self.parameter_names.iter().skip(head) {
            write!(fmt, "{:?}: {:?},", name, variable.type_declaration)?;
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
