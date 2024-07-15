use std::rc::Rc;

use crate::program::functions::{FunctionHead, FunctionInterface, FunctionRepresentation, Parameter, ParameterKey};
use crate::program::traits::{FieldHint, Trait};
use crate::program::types::TypeProto;

pub fn make(name: &str, self_type: &Rc<TypeProto>, field_type: &Rc<TypeProto>, add_getter: bool, add_setter: bool) -> FieldHint {
    let getter = add_getter.then_some({
        let head = FunctionHead::new_static(
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
            FunctionRepresentation::new_member_implicit(name),
        );
        head
    });

    let setter = add_setter.then_some({
        let head = FunctionHead::new_static(
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
            FunctionRepresentation::new_member_implicit(name),
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

pub fn add_to_trait(trait_: &mut Trait, field: FieldHint) {
    if let Some(getter) = &field.getter {
        trait_.abstract_functions.insert(Rc::clone(getter));
    }
    if let Some(setter) = &field.setter {
        trait_.abstract_functions.insert(Rc::clone(setter));
    }
    trait_.field_hints.push(field)
}
