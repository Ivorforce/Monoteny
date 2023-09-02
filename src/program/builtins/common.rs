use std::collections::HashMap;
use std::rc::Rc;
use crate::program::functions::{FunctionInterface, FunctionPointer};
use crate::program::global::BuiltinFunctionHint;
use crate::program::module::Module;
use crate::program::primitives;
use crate::program::traits::Trait;
use crate::program::types::TypeProto;


pub fn create_functions(module: &mut Module, primitive_types: &HashMap<primitives::Type, Rc<Trait>>) {
    let bool_type = TypeProto::simple_struct(&primitive_types[&primitives::Type::Bool]);

    let true_ = FunctionPointer::new_constant(
        "true",
        FunctionInterface::new_constant(&bool_type, vec![])
    );
    module.add_function(&true_);
    module.builtin_hints.insert(
        Rc::clone(&true_),
        BuiltinFunctionHint::True,
    );

    let false_ = FunctionPointer::new_constant(
        "false",
        FunctionInterface::new_constant(&bool_type, vec![])
    );
    module.add_function(&false_);
    module.builtin_hints.insert(
        Rc::clone(&false_),
        BuiltinFunctionHint::False,
    );
}
