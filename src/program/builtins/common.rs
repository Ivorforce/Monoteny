use std::rc::Rc;
use uuid::Uuid;
use crate::program::builtins::core::Core;
use crate::program::functions::{FunctionInterface, FunctionPointer};
use crate::program::module::Module;
use crate::program::primitives;
use crate::program::traits::TraitConformanceRequirement;
use crate::program::types::{TypeProto, TypeUnit};

pub struct Common {
    pub module: Rc<Module>,
    pub true_: Rc<FunctionPointer>,
    pub false_: Rc<FunctionPointer>,
}


pub fn create(core: &Core) -> Common {
    let mut module = Module::new("monoteny.common".into());
    let bool_type = TypeProto::simple_struct(&core.primitives[&primitives::Type::Bool]);

    let true_ = FunctionPointer::new_static(FunctionInterface::new_constant("true", &bool_type, vec![]));
    module.functions.insert(Rc::clone(&true_));

    let false_ = FunctionPointer::new_static(FunctionInterface::new_constant("false", &bool_type, vec![]));
    module.functions.insert(Rc::clone(&false_));

    Common {
        module: Rc::new(module),
        true_, false_
    }
}
