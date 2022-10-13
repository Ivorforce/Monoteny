use std::rc::Rc;
use uuid::Uuid;
use crate::linker::scopes;
use crate::program::allocation::Reference;
use crate::program::functions::FunctionPointer;
use crate::program::structs::Struct;
use crate::program::types::{TypeProto, TypeUnit};


pub struct Strings {
    pub String: Rc<Struct>,
}


pub fn make(constants: &mut scopes::Scope) -> Strings {
    let add_struct = |constants: &mut scopes::Scope, name: &str| -> Rc<Struct> {
        let name = String::from(name);

        let s = Rc::new(Struct {
            id: Uuid::new_v4(),
            name: name.clone(),
        });
        let s_type = TypeProto::meta(TypeProto::unit(TypeUnit::Struct(Rc::clone(&s))));

        constants.insert_singleton(
            scopes::Environment::Global,
            Reference::make_immutable(s_type),
            &name
        );

        s
    };

    Strings {
        String: add_struct(constants, "String")
    }
}
