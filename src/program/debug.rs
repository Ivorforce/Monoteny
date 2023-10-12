use std::fmt::{Display, Formatter};
use itertools::Itertools;
use uuid::Uuid;
use crate::program::computation_tree::ExpressionID;
use crate::program::function_object::{FunctionForm, FunctionRepresentation};
use crate::program::functions::{FunctionInterface, Parameter, ParameterKey};
use crate::program::generics::TypeForest;
use crate::program::types::{TypeProto, TypeUnit};

pub struct MockFunctionInterface<'a> {
    pub representation: FunctionRepresentation,
    pub argument_keys: Vec<ParameterKey>,
    pub arguments: Vec<ExpressionID>,
    pub types: &'a TypeForest,
}

impl<'a> Display for MockFunctionInterface<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let signature = FunctionInterface {
            parameters: self.argument_keys.iter().zip(&self.arguments).map(|(key, expression_id)| Parameter {
                external_key: (*key).clone(),
                internal_name: match key {
                    ParameterKey::Positional => "_".to_string(),
                    ParameterKey::Name(n) => n.clone(),
                },
                type_: self.types.prototype_binding_alias(expression_id),
            }).collect_vec(),
            return_type: TypeProto::unit(TypeUnit::Generic(Uuid::new_v4())),
            requirements: Default::default(),
            generics: Default::default(),
        };
        signature.format(f, &self.representation)
    }
}
