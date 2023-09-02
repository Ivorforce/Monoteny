use std::rc::Rc;
use uuid::Uuid;
use crate::program::functions::{Function, FunctionCallType, FunctionForm, FunctionInterface, FunctionPointer};
use crate::program::global::BuiltinFunctionHint;
use crate::program::module::Module;
use crate::program::types::{TypeProto, TypeUnit};

pub struct Debug {
    pub module: Rc<Module>,
    pub print: Rc<FunctionPointer>,
    pub panic: Rc<FunctionPointer>,
}

pub fn create() -> Debug {
    let mut module = Module::new("monoteny.debug".into());

    let generic_id = Uuid::new_v4();
    let generic_type = TypeProto::unit(TypeUnit::Generic(generic_id));

    let print_function = Rc::new(FunctionPointer {
        pointer_id: Uuid::new_v4(),
        target: Function::new(FunctionInterface::new_simple([generic_type.clone()].into_iter(), TypeProto::void())),
        call_type: FunctionCallType::Static,
        name: "print".into(),
        form: FunctionForm::Global,
    });
    module.add_function(&print_function);
    module.builtin_hints.insert(Rc::clone(&print_function), BuiltinFunctionHint::Print);

    let panic_function = Rc::new(FunctionPointer {
        pointer_id: Uuid::new_v4(),
        target: Function::new(FunctionInterface::new_simple([].into_iter(), generic_type.clone())),
        call_type: FunctionCallType::Static,
        name: "panic".into(),
        form: FunctionForm::Global,
    });
    module.add_function(&panic_function);
    module.builtin_hints.insert(Rc::clone(&panic_function), BuiltinFunctionHint::Panic);

    Debug {
        module: Rc::new(module),
        print: print_function,
        panic: panic_function,
    }
}
