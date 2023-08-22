use std::rc::Rc;
use crate::program::builtins::core::Core;
use crate::program::functions::{FunctionInterface, FunctionPointer};
use crate::program::module::Module;
use crate::program::types::TypeProto;


// TODO This module should be written in monoteny.
pub struct Math {
    pub module: Rc<Module>,
    pub pi: Rc<FunctionPointer>,
    pub tau: Rc<FunctionPointer>,
    pub e: Rc<FunctionPointer>,
}


pub fn create(core: &Core) -> Math {
    let mut module = Module::new("monoteny.math".into());

    let float_generic = TypeProto::make_any();
    let float_requirement = core.traits.Float.create_generic_binding(vec![(&"self".into(), float_generic.clone())]);

    // TODO We should also provide builtin implementations for these (call to from_literal)

    let pi = FunctionPointer::new_constant(
        "pi",
        FunctionInterface::new_constant(&float_generic, vec![&float_requirement])
    );
    module.add_function(&pi);

    let tau = FunctionPointer::new_constant(
        "tau",
        FunctionInterface::new_constant(&float_generic, vec![&float_requirement])
    );
    module.add_function(&tau);

    let e = FunctionPointer::new_constant(
        "e",
        FunctionInterface::new_constant(&float_generic, vec![&float_requirement])
    );
    module.add_function(&e);

    Math {
        module: Rc::new(module),
        pi, tau, e,
    }
}
