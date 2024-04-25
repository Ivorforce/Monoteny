use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use display_with_options::{DebugWithOptions, DisplayWithOptions, with_options};

use uuid::Uuid;

use crate::program::function_object::{FunctionCallExplicity, FunctionRepresentation, FunctionTargetType};
use crate::program::traits::{Trait, TraitBinding};
use crate::program::types::TypeProto;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ParameterKey {
    Positional,
    Name(String),
}

#[derive(Clone, PartialEq, Eq)]
pub enum FunctionType {
    /// A normal function.
    Static,
    /// Not a real function; instead, it refers to a function of some requirement.
    Polymorphic { assumed_requirement: Rc<TraitBinding>, abstract_function: Rc<FunctionHead> },
}

/// The 'head' of a function. It is identifiable by its ID and has an interface.
/// Could be abstract or implemented, depending on whether an implementation is provided!
/// It can also be polymorphic depending on the function_type.
pub struct FunctionHead {
    pub function_id: Uuid,
    pub function_type: FunctionType,
    pub interface: Rc<FunctionInterface>,
}

/// A parameter as visible from the outside.
/// They are expected to be passed in order, and will only be assigned to variables
/// per implementation.
#[derive(Clone, PartialEq, Eq)]
pub struct Parameter {
    pub external_key: ParameterKey,
    pub internal_name: String,
    pub type_: Rc<TypeProto>,
}

/// Machine interface of the function. Everything needed to call it.
#[derive(Clone, PartialEq, Eq)]
pub struct FunctionInterface {
    /// Parameters to the function
    pub parameters: Vec<Parameter>,
    /// Type of what the function returns
    pub return_type: Rc<TypeProto>,

    /// Requirements for parameters and the return type.
    pub requirements: HashSet<Rc<TraitBinding>>,
    /// All internally used generics. These are not guaranteed to not exist elsewhere,
    /// but for the purposes of this interface, they are to be regarded as generics.
    pub generics: HashMap<String, Rc<Trait>>,
}

impl FunctionInterface {
    pub fn new_provider<'a>(return_type: &Rc<TypeProto>, requirements: Vec<&Rc<TraitBinding>>) -> Rc<FunctionInterface> {
        Rc::new(FunctionInterface {
            parameters: vec![],
            return_type: return_type.clone(),
            requirements: requirements.into_iter().map(Rc::clone).collect(),
            generics: Default::default(),
        })
    }

    pub fn new_consumer<'a>(parameter_type: &Rc<TypeProto>, requirements: Vec<&Rc<TraitBinding>>) -> Rc<FunctionInterface> {
        Rc::new(FunctionInterface {
            parameters: vec![Parameter {
                external_key: ParameterKey::Positional,
                internal_name: "arg".to_string(),
                type_: parameter_type.clone(),
            }],
            return_type: TypeProto::void(),
            requirements: requirements.into_iter().map(Rc::clone).collect(),
            generics: Default::default(),
        })
    }

    pub fn new_operator<'a>(count: usize, parameter_type: &Rc<TypeProto>, return_type: &Rc<TypeProto>) -> Rc<FunctionInterface> {
        let parameters: Vec<Parameter> = (0..count)
            .map(|x| { Parameter {
                external_key: ParameterKey::Positional,
                internal_name: format!("p{}", x),
                type_: parameter_type.clone(),
            }
        }).collect();

        Rc::new(FunctionInterface {
            parameters,
            return_type: return_type.clone(),
            requirements: Default::default(),
            generics: Default::default(),
        })
    }

    pub fn new_simple<'a, I>(parameter_types: I, return_type: Rc<TypeProto>) -> Rc<FunctionInterface> where I: Iterator<Item=Rc<TypeProto>> {
        let parameters: Vec<Parameter> = parameter_types
            .enumerate()
            .map(|(i, x)| Parameter {
                external_key: ParameterKey::Positional,
                internal_name: format!("p{}", i),
                type_: x.clone(),
            })
            .collect();

        Rc::new(FunctionInterface {
            parameters,
            return_type: return_type.clone(),
            requirements: Default::default(),
            generics: Default::default(),
        })
    }

    pub fn new_member<'a, I>(self_type: Rc<TypeProto>, parameter_types: I, return_type: Rc<TypeProto>) -> Rc<FunctionInterface> where I: Iterator<Item=Rc<TypeProto>> {
        let parameters: Vec<Parameter> = [Parameter {
                external_key: ParameterKey::Positional,
                internal_name: "self".to_string(),
                type_: self_type,
            }].into_iter().chain(parameter_types
            .enumerate()
            .map(|(i, x)| Parameter {
                external_key: ParameterKey::Positional,
                internal_name: format!("p{}", i),
                type_: x.clone(),
            }))
            .collect();

        Rc::new(FunctionInterface {
            parameters,
            return_type: return_type.clone(),
            requirements: Default::default(),
            generics: Default::default(),
        })
    }
}

impl FunctionHead {
    pub fn new_static(interface: Rc<FunctionInterface>) -> Rc<FunctionHead> {
        Self::new(interface, FunctionType::Static)
    }

    pub fn new(interface: Rc<FunctionInterface>, function_type: FunctionType) -> Rc<FunctionHead> {
        Rc::new(FunctionHead {
            function_id: Uuid::new_v4(),
            interface,
            function_type
        })
    }

    pub fn unwrap_id(&self) -> Uuid {
        match &self.function_type {
            FunctionType::Static => self.function_id,
            FunctionType::Polymorphic { .. } => panic!("Cannot unwrap polymorphic implementation ID"),
        }
    }
}

impl Parameter {
    pub fn mapping_type(&self,  map: &dyn Fn(&Rc<TypeProto>) -> Rc<TypeProto>) -> Parameter {
        Parameter {
            external_key: self.external_key.clone(),
            internal_name: self.internal_name.clone(),
            type_: map(&self.type_),
        }
    }
}

impl PartialEq for FunctionHead {
    fn eq(&self, other: &Self) -> bool {
        self.function_id == other.function_id
    }
}

impl Eq for FunctionHead {}

impl Hash for FunctionHead {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.function_id.hash(state);
    }
}

impl Debug for FunctionHead {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{:?}", with_options(self, &FunctionRepresentation::new("fn", FunctionTargetType::Global, FunctionCallExplicity::Explicit)))
    }
}

impl Display for ParameterKey {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterKey::Name(s) => write!(fmt, "{}: ", s),
            ParameterKey::Positional => Ok(()),
        }
    }
}

impl DebugWithOptions<FunctionRepresentation> for FunctionHead {
    fn fmt(&self, fmt: &mut Formatter<'_>, representation: &FunctionRepresentation) -> std::fmt::Result {
        let call_type_symbol = match self.function_type {
            FunctionType::Static => "|",
            FunctionType::Polymorphic { .. } => "?"
        };
        write!(fmt, "-{}({})--> {:?}", call_type_symbol, &self.function_id, with_options(self.interface.as_ref(), representation))
    }
}

impl DebugWithOptions<FunctionRepresentation> for FunctionInterface {
    fn fmt(&self, fmt: &mut Formatter<'_>, representation: &FunctionRepresentation) -> std::fmt::Result {
        fn format_parameter(fmt: &mut Formatter, parameter: &Parameter) -> std::fmt::Result {
            match &parameter.external_key {
                ParameterKey::Positional => {
                    write!(fmt, "{} '{:?},", parameter.internal_name, parameter.type_)?;
                }
                ParameterKey::Name(n) => {
                    if n != &parameter.internal_name {
                        write!(fmt, "{}: {} '{:?},", n, parameter.internal_name, parameter.type_)?;
                    } else {
                        write!(fmt, "{}: '{:?},", n, parameter.type_)?;
                    }
                }
            }
            Ok(())
        }

        let mut head = 0;

        if representation.target_type == FunctionTargetType::Member {
            write!(fmt, "(")?;
            format_parameter(fmt, self.parameters.get(head).unwrap())?;
            write!(fmt, ").")?;
            head += 1;
        }

        write!(fmt, "{}", representation.name)?;

        if representation.call_explicity == FunctionCallExplicity::Explicit {
            write!(fmt, "(")?;
            for parameter in self.parameters.iter().skip(head) {
                format_parameter(fmt, &parameter)?;
            }
            write!(fmt, ")")?;
        }

        if !self.return_type.unit.is_void() {
            write!(fmt, " -> {:?}", self.return_type)?;
        }

        Ok(())
        // TODO Requirements?
    }
}
