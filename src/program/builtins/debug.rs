use std::rc::Rc;
use uuid::Uuid;
use crate::linker::scopes;
use crate::program::functions::{FunctionInterface, FunctionPointer};
use crate::program::types::{TypeProto, TypeUnit};

pub struct Functions {
    pub print: Rc<FunctionPointer>,
}

pub fn make_functions(constants: &mut scopes::Scope) -> Functions {
    let generic_id = Uuid::new_v4();
    let generic_type = TypeProto::unit(TypeUnit::Any(generic_id));

    let print_function = FunctionPointer::new_static(
        FunctionInterface::new_global("print", [generic_type.clone()].into_iter(), TypeProto::void())
    );
    constants.overload_function(&print_function);

    Functions {
        print: print_function,
    }
}
