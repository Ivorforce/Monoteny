use std::rc::Rc;
use uuid::Uuid;
use crate::program::builtins::core::Core;
use crate::program::functions::{FunctionInterface, FunctionPointer};
use crate::program::module::Module;
use crate::program::traits::{Trait};
use crate::program::types::{TypeProto, TypeUnit};


pub struct Transpilation {
    pub module: Rc<Module>,
    pub any_type: Box<TypeProto>,

    pub Transpiler: Rc<Trait>,
    pub add: Rc<FunctionPointer>,
}

pub fn create(core: &Core) -> Transpilation {
    let mut module = Module::new("monoteny.transpiler".into());

    let any_type = TypeProto::unit(TypeUnit::Any(Uuid::new_v4()));

    let mut Transpiler = Trait::new("Transpiler".into());
    let Transpiler = Rc::new(Transpiler);
    module.add_trait(&Transpiler);

    let add = FunctionPointer::new_member(
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
    module.add_function(&add);

    Transpilation {
        module: Rc::new(module),
        any_type,

        Transpiler, add
    }
}
