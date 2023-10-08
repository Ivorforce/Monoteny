use std::rc::Rc;
use uuid::Uuid;
use crate::program::builtins::core::Core;
use crate::program::functions::{FunctionInterface, FunctionPointer};
use crate::program::module::Module;
use crate::program::traits::{Trait};
use crate::program::types::{TypeProto, TypeUnit};


#[allow(non_snake_case)]
pub struct Transpilation {
    pub module: Rc<Module>,
    pub any_type: Box<TypeProto>,

    pub Transpiler: Rc<Trait>,
    pub add: Rc<FunctionPointer>,
}

#[allow(non_snake_case)]
pub fn create(core: &Core) -> Transpilation {
    let mut module = Module::new("monoteny.transpiler".to_string());

    let any_type = TypeProto::unit(TypeUnit::Generic(Uuid::new_v4()));

    let mut Transpiler = Trait::new_with_self("Transpiler".to_string());
    let Transpiler = Rc::new(Transpiler);
    module.add_trait(&Transpiler);

    let add = FunctionPointer::new_member_function(
        "add",
        FunctionInterface::new_simple(
            [
                TypeProto::unit(TypeUnit::Struct(Rc::clone(&Transpiler))),
                // TODO This should be 'any function' but there's no need to implement that until we
                //  need function abstractions in the future otherwise.
                any_type.clone()
            ].into_iter(),
            TypeProto::unit(TypeUnit::Void),
        )
    );
    module.add_function(Rc::clone(&add));

    Transpilation {
        module: Rc::new(module),
        any_type,

        Transpiler, add
    }
}
