use std::rc::Rc;
use uuid::Uuid;
use crate::linker::scopes::Scope;
use crate::program::builtins::traits::Traits;
use crate::program::functions::{FunctionInterface, FunctionPointer};
use crate::program::primitives;
use crate::program::traits::TraitConformanceRequirement;
use crate::program::types::{TypeProto, TypeUnit};

pub struct Common {
    pub true_: Rc<FunctionPointer>,
    pub false_: Rc<FunctionPointer>,
}


pub fn make(mut constants: &mut Scope, traits: &Traits) -> Common {
    let bool_type = TypeProto::unit(TypeUnit::Primitive(primitives::Type::Bool));

    let true_ = FunctionPointer::new_static(FunctionInterface::new_constant("true", &bool_type, vec![]));
    constants.overload_function(&true_);

    let false_ = FunctionPointer::new_static(FunctionInterface::new_constant("false", &bool_type, vec![]));
    constants.overload_function(&false_);

    Common {
        true_, false_
    }
}
