use std::collections::HashSet;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use uuid::Uuid;
use crate::LinkError;
use crate::program::allocation::ObjectReference;
use crate::program::traits::{TraitBinding};
use crate::program::types::{TypeProto, TypeUnit};

#[derive(Clone, PartialEq, Eq, Hash)]
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
pub enum FunctionType {
    Static,
    /// Not a real function call, rather to be delegated through the requirement's resolution.
    Polymorphic { provided_by_assumption: Rc<TraitBinding>, abstract_function: Rc<FunctionPointer> },
}

/// The 'head' of a function. It is identifiable by its ID and has an interface.
/// Could be abstract or implemented, depending on whether an implementation is provided!
/// It can also be polymorphic depending on the function_type.
pub struct FunctionHead {
    pub function_id: Uuid,
    pub function_type: FunctionType,
    pub interface: Rc<FunctionInterface>,
}

/// An object that says 'Oh, I know a function!'
/// It associates the function with a name and a form.
#[derive(PartialEq, Eq, Hash)]
pub struct FunctionPointer {
    /// The underlying function.
    pub target: Rc<FunctionHead>,

    /// Name of the function.
    pub name: String,
    /// How the functon looks in syntax.
    pub form: FunctionForm,
}

/// Reference to a multiplicity of functions, usually resolved when attempting to call
#[derive(Clone, PartialEq, Eq)]
pub struct FunctionOverload {
    pub pointers: HashSet<Rc<ObjectReference>>,
    pub name: String,
    pub form: FunctionForm,
}

/// A parameter as visible from the outside.
/// They are expected to be passed in order, and will only be assigned to variables
/// per implementation.
#[derive(Clone, PartialEq, Eq)]
pub struct Parameter {
    pub external_key: ParameterKey,
    pub internal_name: String,
    pub type_: Box<TypeProto>,
}

/// Machine interface of the function. Everything needed to call it.
#[derive(Clone, PartialEq, Eq)]
pub struct FunctionInterface {
    /// Parameters to the function
    pub parameters: Vec<Parameter>,
    /// Type of what the function returns
    pub return_type: Box<TypeProto>,

    /// Requirements for parameters and the return type.
    pub requirements: HashSet<Rc<TraitBinding>>,
}

impl FunctionInterface {
    pub fn new_constant<'a>(return_type: &Box<TypeProto>, requirements: Vec<&Rc<TraitBinding>>) -> Rc<FunctionInterface> {
        Rc::new(FunctionInterface {
            parameters: vec![],
            return_type: return_type.clone(),
            requirements: requirements.into_iter().map(Rc::clone).collect(),
        })
    }

    pub fn new_operator<'a>(count: usize, parameter_type: &Box<TypeProto>, return_type: &Box<TypeProto>) -> Rc<FunctionInterface> {
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
        })
    }

    pub fn new_simple<'a, I>(parameter_types: I, return_type: Box<TypeProto>) -> Rc<FunctionInterface> where I: Iterator<Item=Box<TypeProto>> {
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
        })
    }

    pub fn new_member<'a, I>(self_type: Box<TypeProto>, parameter_types: I, return_type: Box<TypeProto>) -> Rc<FunctionInterface> where I: Iterator<Item=Box<TypeProto>> {
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
        })
    }

    pub fn collect_generics(&self) -> HashSet<Uuid> {
        TypeProto::collect_generics(self.parameters.iter().map(|x| &x.type_).chain([&self.return_type]))
    }

    pub fn fmt_with_form(&self, fmt: &mut Formatter<'_>, name: &str, form: &FunctionForm) -> std::fmt::Result {
        let mut head = 0;

        match form {
            FunctionForm::Global => {}
            FunctionForm::Constant => {}
            FunctionForm::Member => {
                write!(fmt, "{{'{:?}}}.", self.parameters.get(head).unwrap().type_)?;
                head += 1;
            },
        }

        write!(fmt, "{}(", name)?;

        for parameter in self.parameters.iter().skip(head) {
            match &parameter.external_key {
                ParameterKey::Positional => {
                    write!(fmt, "{} '{:?},", parameter.internal_name, parameter.type_)?;
                }
                ParameterKey::Name(n) => {
                    write!(fmt, "{}: {} '{:?},", n, parameter.internal_name, parameter.type_)?;
                }
            }
        }

        write!(fmt, ")")?;

        if !self.return_type.unit.is_void() {
            write!(fmt, " -> {:?}", self.return_type)?;
        }

        Ok(())
        // TODO Requirements?
    }
}

impl FunctionPointer {
    pub fn new_global(name: &str, interface: Rc<FunctionInterface>) -> Rc<FunctionPointer> {
        Rc::new(FunctionPointer {
            target: FunctionHead::new(interface, FunctionType::Static),
            name: name.into(),
            form: FunctionForm::Global,
        })
    }

    pub fn new_member(name: &str, interface: Rc<FunctionInterface>) -> Rc<FunctionPointer> {
        Rc::new(FunctionPointer {
            target: FunctionHead::new(interface, FunctionType::Static),
            name: name.into(),
            form: FunctionForm::Member,
        })
    }

    pub fn new_constant(name: &str, interface: Rc<FunctionInterface>) -> Rc<FunctionPointer> {
        Rc::new(FunctionPointer {
            target: FunctionHead::new(interface, FunctionType::Static),
            name: name.into(),
            form: FunctionForm::Constant,
        })
    }

    pub fn can_match_strict(&self, other: &FunctionPointer) -> bool {
        if &self.name != &other.name { return false; }
        if &self.form != &other.form { return false; }

        self.target.can_match(&other.target)
    }
}

impl FunctionHead {
    pub fn new(interface: Rc<FunctionInterface>, function_type: FunctionType) -> Rc<FunctionHead> {
        Rc::new(FunctionHead {
            function_id: Uuid::new_v4(),
            interface,
            function_type
        })
    }

    pub fn fmt_with_form(&self, fmt: &mut Formatter<'_>, name: &str, form: &FunctionForm) -> std::fmt::Result {
        let call_type_symbol = match self.function_type {
            FunctionType::Static => "|",
            FunctionType::Polymorphic { .. } => "?"
        };
        write!(fmt, "-{}({})--> ", call_type_symbol, &self.function_id)?;

        self.interface.fmt_with_form(fmt, name, form)
    }

    pub fn unwrap_id(&self) -> Uuid {
        match &self.function_type {
            FunctionType::Static => self.function_id,
            FunctionType::Polymorphic { .. } => panic!("Cannot unwrap polymorphic implementation ID"),
        }
    }

    pub fn can_match(&self, other: &FunctionHead) -> bool {
        // TODO Should try to match generics?
        if &self.function_type != &other.function_type { return false; }
        if &self.interface != &other.interface { return false; }

        true
    }
}

impl FunctionOverload {
    pub fn from(function: &Rc<FunctionPointer>, object_ref: &Rc<ObjectReference>) -> Rc<FunctionOverload> {
        Rc::new(FunctionOverload {
            pointers: HashSet::from([Rc::clone(object_ref)]),
            name: function.name.clone(),
            form: function.form.clone(),
        })
    }

    pub fn adding_function(&self, function: &Rc<FunctionPointer>, object_ref: &Rc<ObjectReference>) -> Result<Rc<FunctionOverload>, LinkError> {
        if self.form != function.form {
            return Err(LinkError::LinkError { msg: format!("Cannot overload functions and constants.") })
        }

        Ok(Rc::new(FunctionOverload {
            pointers: self.pointers.iter()
                .chain([object_ref])
                .map(Rc::clone)
                .collect(),
            name: self.name.clone(),
            form: self.form.clone(),
        }))
    }

    pub fn functions(&self) -> Vec<Rc<FunctionHead>> {
        self.pointers.iter().map(|x| match &x.type_.unit {
            TypeUnit::Function(f) => Rc::clone(f),
            _ => panic!("Function overload has a non-function!")
        }).collect()
    }
}

impl Parameter {
    pub fn mapping_type(&self,  map: &dyn Fn(&Box<TypeProto>) -> Box<TypeProto>) -> Parameter {
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

impl Debug for FunctionPointer {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        self.target.fmt_with_form(fmt, &self.name, &self.form)
    }
}

impl Debug for FunctionHead {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt_with_form(fmt, &"fn".to_string(), &FunctionForm::Global)
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
