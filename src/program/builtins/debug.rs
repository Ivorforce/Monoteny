use std::rc::Rc;
use uuid::Uuid;
use crate::program::functions::{FunctionInterface, FunctionPointer};
use crate::program::module::Module;
use crate::program::types::{TypeProto, TypeUnit};

pub struct Debug {
    pub module: Rc<Module>,
    pub print: Rc<FunctionPointer>,
}

pub fn create() -> Debug {
    let mut module = Module::new("monoteny.debug".into());

    let generic_id = Uuid::new_v4();
    let generic_type = TypeProto::unit(TypeUnit::Any(generic_id));

    let print_function = FunctionPointer::new_static(
        FunctionInterface::new_global("print", [generic_type.clone()].into_iter(), TypeProto::void())
    );
    module.add_function(&print_function);

    Debug {
        module: Rc::new(module),
        print: print_function,
    }
}
