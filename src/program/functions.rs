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
pub enum FunctionCallType {
    Static { function_id: Uuid },
    Polymorphic { abstract_function: Rc<AbstractFunction> },
}

pub struct FunctionPointer {
    pub pointer_id: Uuid,
    pub call_type: FunctionCallType,
    pub interface: Rc<FunctionInterface>,
}

/// Reference to a multiplicity of functions, usually resolved when attempting to call
#[derive(Clone, PartialEq, Eq)]
pub struct FunctionOverload {
    pub pointers: HashSet<Rc<FunctionPointer>>,
    pub name: String,
    pub form: FunctionForm,
}

pub struct Parameter {
    pub external_key: ParameterKey,
    pub internal_name: String,
    pub target: Rc<ObjectReference>,
}

pub struct AbstractFunction {
    pub function_id: Uuid,
    pub interface: Rc<FunctionInterface>,
}

pub struct FunctionInterface {
    /// Parameters to the function
    pub parameters: Vec<Parameter>,
    /// Type of what the function returns
    pub return_type: Box<TypeProto>,

    /// Name of the function.
    pub name: String,
    /// How the functon looks in syntax.
    pub form: FunctionForm,
    /// Requirements for parameters and the return type.
    pub requirements: HashSet<Rc<TraitConformanceRequirement>>,
}

impl FunctionInterface {
    pub fn new_constant<'a>(alphanumeric_name: &'a str, return_type: &Box<TypeProto>, requirements: Vec<&Rc<TraitConformanceRequirement>>) -> Rc<FunctionInterface> {
        Rc::new(FunctionInterface {
            name: String::from(alphanumeric_name),
            parameters: vec![],
            form: FunctionForm::Constant,
            return_type: return_type.clone(),
            requirements: requirements.into_iter().map(Rc::clone).collect(),
        })
    }

    pub fn new_operator<'a>(alphanumeric_name: &'a str, count: usize, parameter_type: &Box<TypeProto>, return_type: &Box<TypeProto>) -> Rc<FunctionInterface> {
        let parameters: Vec<Parameter> = (0..count)
            .map(|x| { Parameter {
                external_key: ParameterKey::Positional,
                internal_name: format!("p{}", x),
                target: ObjectReference::make_immutable(parameter_type.clone()),
            }
        }).collect();

        Rc::new(FunctionInterface {
            name: String::from(alphanumeric_name),
            parameters,
            form: FunctionForm::Global,
            return_type: return_type.clone(),
            requirements: HashSet::new(),
        })
    }

    pub fn new_global<'a, I>(name: &'a str, parameter_types: I, return_type: Box<TypeProto>) -> Rc<FunctionInterface> where I: Iterator<Item=Box<TypeProto>> {
        let parameters: Vec<Parameter> = parameter_types
            .map(|x| Parameter {
                external_key: ParameterKey::Positional,
                internal_name: format!("p"),  // TODO Should be numbered? idk
                target: ObjectReference::make_immutable(x.clone()),
            })
            .collect();

        Rc::new(FunctionInterface {
            name: String::from(name),
            parameters,
            form: FunctionForm::Global,
            return_type: return_type.clone(),
            requirements: HashSet::new(),
        })
    }

    pub fn new_member<'a, I>(name: &'a str, parameter_types: I, return_type: Box<TypeProto>) -> Rc<FunctionInterface> where I: Iterator<Item=Box<TypeProto>> {
        let parameters: Vec<Parameter> = parameter_types
            .map(|x| Parameter {
                external_key: ParameterKey::Positional,
                internal_name: format!("p"),  // TODO Should be numbered? idk
                target: ObjectReference::make_immutable(x.clone()),
            })
            .collect();

        Rc::new(FunctionInterface {
            name: String::from(name),
            parameters,
            form: FunctionForm::Member,
            return_type: return_type.clone(),
            requirements: HashSet::new(),
        })
    }
}

impl FunctionPointer {
    pub fn new_static(interface: Rc<FunctionInterface>) -> Rc<FunctionPointer> {
        Rc::new(FunctionPointer {
            pointer_id: Uuid::new_v4(),
            call_type: FunctionCallType::Static { function_id: Uuid::new_v4() },
            interface,
        })
    }

    pub fn new_polymorphic(abstract_function: Rc<AbstractFunction>) -> Rc<FunctionPointer> {
        Rc::new(FunctionPointer {
            pointer_id: Uuid::new_v4(),
            interface: Rc::clone(&abstract_function.interface),
            call_type: FunctionCallType::Polymorphic { abstract_function },
        })
    }

    pub fn unwrap_id(&self) -> Uuid {
        match self.call_type {
            FunctionCallType::Static { function_id } => function_id,
            FunctionCallType::Polymorphic { .. } => panic!("Cannot unwrap polymorphic implementation ID"),
        }
    }
}

impl AbstractFunction {
    pub fn new(interface: Rc<FunctionInterface>) -> Rc<AbstractFunction> {
        Rc::new(AbstractFunction {
            function_id: Uuid::new_v4(),
            interface,
        })
    }
}

impl FunctionOverload {
    pub fn from(function: &Rc<FunctionPointer>) -> Rc<FunctionOverload> {
        Rc::new(FunctionOverload {
            pointers: HashSet::from([Rc::clone(function)]),
            name: function.interface.name.clone(),
            form: function.interface.form.clone(),
        })
    }

    pub fn adding_function(&self, function: &Rc<FunctionPointer>) -> Result<Rc<FunctionOverload>, LinkError> {
        if self.form != function.interface.form {
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

impl PartialEq for AbstractFunction {
    fn eq(&self, other: &Self) -> bool {
        self.function_id == other.function_id
    }
}

impl Eq for AbstractFunction {}

impl Hash for AbstractFunction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.function_id.hash(state);
    }
}

impl Debug for FunctionInterface {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        let mut head = 0;

        match self.form {
            FunctionForm::Global => {}
            FunctionForm::Constant => {}
            FunctionForm::Member => {
                write!(fmt, "{:?}.", self.parameters.get(head).unwrap().target.type_)?;
                head += 1;
            },
        }

        write!(fmt, "{}(", self.name)?;

        for parameter in self.parameters.iter().skip(head) {
            write!(fmt, "{:?} '{:?},", parameter.external_key, parameter.target.type_)?;
        }

        write!(fmt, ")")?;

        if !self.return_type.unit.is_void() {
            write!(fmt, " -> {:?}", self.return_type)?;
        }

        Ok(())
    }
}

impl Debug for AbstractFunction {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "abstract {:?}", self.interface)
    }
}

impl Debug for ParameterKey {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        use ParameterKey::*;
        use crate::program::functions::ParameterKey::Positional;
        match self {
            Name(s) => write!(fmt, ":{}", s),
            Positional => write!(fmt, "<>"),
        }
    }
}
