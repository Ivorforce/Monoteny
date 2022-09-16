use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use itertools::zip_eq;
use uuid::Uuid;
use crate::program::traits::TraitConformanceRequirement;
use crate::program::types::{Mutability, ParameterKey, Type, Variable};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FunctionForm {
    Global,
    Member,
    Operator,
}

pub struct FunctionPointer {
    pub pointer_id: Uuid,
    pub function_id: Uuid,

    pub requirements: HashSet<Rc<TraitConformanceRequirement>>,
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
    pub return_type: Option<Box<Type>>,
}

impl MachineFunctionInterface {
    pub fn make_static<I>(parameters: I, return_type: Option<Box<Type>>) -> Rc<MachineFunctionInterface> where I: Iterator<Item=Rc<Variable>> {
        Rc::new(MachineFunctionInterface {
            parameters: parameters.collect(),
            return_type,
        })
    }
}

impl FunctionPointer {
    pub fn make_operator<'a>(name: &'a str, alphanumeric_name: &'a str, count: usize, parameter_type: &Box<Type>, return_type: &Box<Type>) -> Rc<FunctionPointer> {
        let parameter_names = (0..count).map(|_| ParameterKey::Positional);
        let parameters: Vec<Rc<Variable>> = (0..count).map(|x| Rc::new(Variable {
            id: Uuid::new_v4(),
            type_declaration: parameter_type.clone(),
            mutability: Mutability::Immutable
        })).collect();

        Rc::new(FunctionPointer {
            function_id: Uuid::new_v4(),
            pointer_id: Uuid::new_v4(),

            requirements: HashSet::new(),
            human_interface: Rc::new(HumanFunctionInterface {
                name: String::from(name),
                alphanumeric_name: String::from(alphanumeric_name),
                parameter_names: zip_eq(parameter_names, parameters.iter().map(|x| Rc::clone(x))).collect(),
                parameter_names_internal: vec![],  // TODO Internal names shouldn't need to be specified for builtins?
                form: FunctionForm::Operator,
            }),
            machine_interface: MachineFunctionInterface::make_static(parameters.into_iter(), Some(return_type.clone()))
        })
    }

    pub fn make_global<'a, I>(name: &'a str, alphanumeric_name: &'a str, parameter_types: I, return_type: Option<Box<Type>>) -> Rc<FunctionPointer> where I: Iterator<Item=Box<Type>> {
        let parameters: Vec<Rc<Variable>> = parameter_types.map(|x| Rc::new(Variable {
            id: Uuid::new_v4(),
            type_declaration: x.clone(),
            mutability: Mutability::Immutable
        })).collect();
        let parameter_names = (0..parameters.len()).map(|_| ParameterKey::Positional);

        Rc::new(FunctionPointer {
            function_id: Uuid::new_v4(),
            pointer_id: Uuid::new_v4(),

            requirements: HashSet::new(),
            human_interface: Rc::new(HumanFunctionInterface {
                name: String::from(name),
                alphanumeric_name: String::from(alphanumeric_name),
                parameter_names: zip_eq(parameter_names, parameters.iter().map(|x| Rc::clone(x))).collect(),
                parameter_names_internal: vec![],  // TODO Internal names shouldn't need to be specified for builtins?
                form: FunctionForm::Global,
            }),
            machine_interface: MachineFunctionInterface::make_static(parameters.into_iter(), return_type)
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