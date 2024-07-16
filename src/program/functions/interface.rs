use std::collections::HashSet;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

use display_with_options::{DebugWithOptions, with_options};

use crate::program::functions::{FunctionCallExplicity, FunctionRepresentation, FunctionTargetType};
use crate::program::traits::{Trait, TraitBinding};
use crate::program::types::TypeProto;
use crate::util::fmt::write_separated_debug;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ParameterKey {
    Positional,
    Name(String),
}

/// A parameter as visible from the outside.
/// They are expected to be passed in order, and will only be assigned to variables
/// per implementation.
#[derive(Clone, PartialEq, Eq)]
pub struct Parameter {
    pub external_key: ParameterKey,
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
    pub generics: HashSet<Rc<Trait>>,
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
                type_: self_type,
            }].into_iter().chain(parameter_types
            .enumerate()
            .map(|(i, x)| Parameter {
                external_key: ParameterKey::Positional,
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

impl Parameter {
    pub fn mapping_type(&self,  map: &dyn Fn(&Rc<TypeProto>) -> Rc<TypeProto>) -> Parameter {
        Parameter {
            external_key: self.external_key.clone(),
            type_: map(&self.type_),
        }
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

impl Debug for Parameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.external_key {
            ParameterKey::Positional => {
                write!(f, "_ '{:?}", self.type_)
            }
            ParameterKey::Name(n) => {
                write!(f, "{}: '{:?}", n, self.type_)
            }
        }
    }
}

impl DebugWithOptions<FunctionRepresentation> for FunctionInterface {
    fn fmt(&self, fmt: &mut Formatter<'_>, representation: &FunctionRepresentation) -> std::fmt::Result {
        let mut head = 0;

        if representation.target_type == FunctionTargetType::Member {
            write!(fmt, "({:?}).", self.parameters.get(head).unwrap())?;
            head += 1;
        }

        write!(fmt, "{}", representation.name)?;

        if representation.call_explicity == FunctionCallExplicity::Explicit {
            write!(fmt, "(")?;
            write_separated_debug(fmt, ", ", self.parameters.iter().skip(head))?;
            write!(fmt, ")")?;
        }

        if !self.return_type.unit.is_void() {
            write!(fmt, " -> {:?}", self.return_type)?;
        }

        Ok(())
        // TODO Requirements?
    }
}

impl Debug for FunctionInterface {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", with_options(self, &FunctionRepresentation::dummy()))
    }
}
