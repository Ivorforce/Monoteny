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

    let true_ = FunctionPointer::new_global_implicit(
        "true",
        FunctionInterface::new_provider(&bool_type, vec![])
    );
    module.fn_builtin_hints.insert(
        Rc::clone(&true_.target),
        BuiltinFunctionHint::True,
    );
    module.add_function(true_);

    let false_ = FunctionPointer::new_global_implicit(
        "false",
        FunctionInterface::new_provider(&bool_type, vec![])
    );
    module.fn_builtin_hints.insert(
        Rc::clone(&false_.target),
        BuiltinFunctionHint::False,
    );
    module.add_function(false_);
}
