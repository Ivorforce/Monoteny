use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use uuid::Uuid;
use crate::LinkError;
use crate::program::allocation::ObjectReference;
use crate::program::traits::{TraitBinding};
use crate::program::types::{TypeProto, TypeUnit};

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
    Static,
    /// Not a real function call, rather to be delegated through the requirement's resolution.
    Polymorphic { requirement: Rc<TraitBinding>, abstract_function: Rc<FunctionPointer> },
}

/// A plain, static function that converts a number of parameters to a return type.
/// The presentation (name, form etc.) are given in FunctionPointer.
/// Could be abstract or implemented, depending on whether an implementation is provided!
pub struct Function {
    pub function_id: Uuid,
    pub interface: Rc<FunctionInterface>,
}

/// An object that says 'Oh, I know a function!'
/// It associates the function with a name and a form.
pub struct FunctionPointer {
    pub pointer_id: Uuid,

    /// The underlying function.
    pub target: Rc<Function>,
    pub call_type: FunctionCallType,

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
pub struct Parameter {
    pub external_key: ParameterKey,
    pub internal_name: String,
    pub type_: Box<TypeProto>,
}

/// Machine interface of the function.
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
            .map(|x| Parameter {
                external_key: ParameterKey::Positional,
                internal_name: format!("p"),  // TODO Should be numbered? idk
                type_: x.clone(),
            })
            .collect();

        Rc::new(FunctionInterface {
            parameters,
            return_type: return_type.clone(),
            requirements: Default::default(),
        })
    }

    pub fn collect_anys(&self) -> HashSet<Uuid> {
        TypeProto::collect_anys(self.parameters.iter().map(|x| &x.type_).chain([&self.return_type]))
    }
}

impl FunctionPointer {
    pub fn new_global(name: &str, interface: Rc<FunctionInterface>) -> Rc<FunctionPointer> {
        Rc::new(FunctionPointer {
            pointer_id: Uuid::new_v4(),
            target: Function::new(interface),
            call_type: FunctionCallType::Static,
            name: name.into(),
            form: FunctionForm::Global,
        })
    }

    pub fn new_member(name: &str, interface: Rc<FunctionInterface>) -> Rc<FunctionPointer> {
        Rc::new(FunctionPointer {
            pointer_id: Uuid::new_v4(),
            target: Function::new(interface),
            call_type: FunctionCallType::Static,
            name: name.into(),
            form: FunctionForm::Member,
        })
    }

    pub fn new_constant(name: &str, interface: Rc<FunctionInterface>) -> Rc<FunctionPointer> {
        Rc::new(FunctionPointer {
            pointer_id: Uuid::new_v4(),
            target: Function::new(interface),
            call_type: FunctionCallType::Static,
            name: name.into(),
            form: FunctionForm::Constant,
        })
    }

    pub fn unwrap_id(&self) -> Uuid {
        match &self.call_type {
            FunctionCallType::Static => self.target.function_id,
            FunctionCallType::Polymorphic { .. } => panic!("Cannot unwrap polymorphic implementation ID"),
        }
    }
}

impl Function {
    pub fn new(interface: Rc<FunctionInterface>) -> Rc<Function> {
        Rc::new(Function {
            function_id: Uuid::new_v4(),
            interface,
        })
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

    pub fn functions(&self) -> Vec<Rc<FunctionPointer>> {
        self.pointers.iter().map(|x| match &x.type_.unit {
            TypeUnit::Function(f) => Rc::clone(f),
            _ => panic!("Function overload has a non-function!")
        }).collect()
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

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.function_id == other.function_id
    }
}

impl Eq for Function {}

impl Hash for Function {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.function_id.hash(state);
    }
}

impl Debug for FunctionPointer {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        let mut head = 0;

        let call_type_symbol = match self.call_type {
            FunctionCallType::Static => "|",
            FunctionCallType::Polymorphic { .. } => "?"
        };
        write!(fmt, "-{}({})--> ", call_type_symbol, &self.pointer_id)?;

        match self.form {
            FunctionForm::Global => {}
            FunctionForm::Constant => {}
            FunctionForm::Member => {
                write!(fmt, "{{'{:?}}}.", self.target.interface.parameters.get(head).unwrap().type_)?;
                head += 1;
            },
        }

        write!(fmt, "{}(", self.name)?;

        for parameter in self.target.interface.parameters.iter().skip(head) {
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

        if !self.target.interface.return_type.unit.is_void() {
            write!(fmt, " -> {:?}", self.target.interface.return_type)?;
        }

        Ok(())
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
