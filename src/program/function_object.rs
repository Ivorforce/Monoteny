use std::collections::HashSet;
use std::rc::Rc;

use crate::error::RResult;
use crate::program::functions::FunctionHead;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum FunctionForm {
    GlobalFunction,
    GlobalImplicit,
    MemberFunction,
    MemberImplicit,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct FunctionRepresentation {
    /// Name of the function.
    pub name: String,
    /// How the functon looks in syntax.
    pub form: FunctionForm,
}

/// Reference to a multiplicity of functions, usually resolved when attempting to call
#[derive(Clone, PartialEq, Eq)]
pub struct FunctionOverload {
    pub functions: HashSet<Rc<FunctionHead>>,
    // Note: If representation is NOT an implicit, the functions within are getters.
    pub representation: FunctionRepresentation,
}

impl FunctionRepresentation {
    pub fn new(name: &str, form: FunctionForm) -> FunctionRepresentation {
        FunctionRepresentation {
            name: name.into(),
            form,
        }
    }
}

impl FunctionOverload {
    pub fn from(function: &Rc<FunctionHead>, representation: FunctionRepresentation) -> Rc<FunctionOverload> {
        Rc::new(FunctionOverload {
            functions: HashSet::from([Rc::clone(function)]),
            representation,
        })
    }

    pub fn adding_function(&self, function: &Rc<FunctionHead>) -> RResult<Rc<FunctionOverload>> {
        Ok(Rc::new(FunctionOverload {
            functions: self.functions.iter()
                .chain([function])
                .map(Rc::clone)
                .collect(),
            representation: self.representation.clone(),
        }))
    }
}

impl FunctionForm {
    pub fn implicit(&self) -> FunctionForm {
        match self {
            FunctionForm::GlobalFunction => FunctionForm::GlobalImplicit,
            FunctionForm::GlobalImplicit => FunctionForm::GlobalImplicit,
            FunctionForm::MemberFunction => FunctionForm::MemberImplicit,
            FunctionForm::MemberImplicit => FunctionForm::MemberImplicit,
        }
    }

    pub fn is_implicit(&self) -> bool {
        match self {
            FunctionForm::GlobalFunction => false,
            FunctionForm::GlobalImplicit => true,
            FunctionForm::MemberFunction => false,
            FunctionForm::MemberImplicit => true,
        }
    }
}
