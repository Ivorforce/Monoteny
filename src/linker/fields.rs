use std::rc::Rc;
use crate::program::functions::{FunctionForm, FunctionHead, FunctionInterface, FunctionPointer, FunctionType, Parameter, ParameterKey};
use crate::program::traits::{Trait, FieldHint};
use crate::program::types::TypeProto;

pub fn make(name: &str, self_type: &Box<TypeProto>, field_type: &Box<TypeProto>, add_getter: bool, add_setter: bool) -> FieldHint {
    let getter = add_getter.then_some({
        let head = FunctionHead::new(
            Rc::new(FunctionInterface {
                parameters: vec![
                    Parameter {
                        external_key: ParameterKey::Positional,
                        internal_name: "self".to_string(),
                        type_: self_type.clone(),
                    }],
                return_type: field_type.clone(),
                requirements: Default::default(),
                generics: Default::default(),
            }),
            FunctionType::Static
        );
        head
    });

    let setter = add_setter.then_some({
        let head = FunctionHead::new(
            Rc::new(FunctionInterface {
                parameters: vec![Parameter {
                    external_key: ParameterKey::Positional,
                    internal_name: "self".to_string(),
                    type_: self_type.clone(),
                }, Parameter {
                    external_key: ParameterKey::Positional,
                    internal_name: name.to_string(),
                    type_: field_type.clone(),
                }],
                return_type: TypeProto::void(),
                requirements: Default::default(),
                generics: Default::default(),
            }),
            FunctionType::Static
        );
        head
    });

    FieldHint {
        name: name.to_string(),
        type_: field_type.clone(),
        setter,
        getter,
    }
}

pub fn add_to_trait(trait_: &mut Trait, hint: FieldHint) {
    if let Some(getter) = &hint.getter {
        trait_.insert_function(
            make_ptr(&hint, Rc::clone(getter))
        )
    }
    if let Some(setter) = &hint.setter {
        trait_.insert_function(
            make_ptr(&hint, Rc::clone(setter))
        )
    }
    trait_.field_hints.push(hint)
}

pub fn make_ptr(hint: &FieldHint, function: Rc<FunctionHead>) -> Rc<FunctionPointer> {
    Rc::new(FunctionPointer {
        target: function,
        name: hint.name.clone(),
        form: FunctionForm::MemberImplicit,
    })
}
