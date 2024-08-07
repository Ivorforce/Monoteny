use std::collections::HashMap;
use std::rc::Rc;

use uuid::Uuid;

use crate::program::expression_tree::ExpressionOperation;
use crate::program::functions::{FunctionHead, FunctionImplementation};
use crate::program::types::TypeProto;
use crate::transpiler::namespaces;

#[derive(Clone)]
pub struct Representations {
    pub function_forms: HashMap<Rc<FunctionHead>, FunctionForm>,
    pub type_ids: HashMap<Rc<TypeProto>, Uuid>,
}

impl Representations {
    pub fn new() -> Representations {
        Representations {
            function_forms: Default::default(),
            type_ids: Default::default(),
        }
    }
}

// The IDs are attached per object because theoretically it's possible for a representation to use
//  0 names (direct keyword use) or 2 (using multiple keywords). They just 'happen' to all use one.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum FunctionForm {
    Identity,
    CallAsFunction,
    Constant(Uuid),
    FunctionCall(Uuid),
    SetMemberField(Uuid),
    GetMemberField(Uuid),
    MemberCall(Uuid),
    Unary(Uuid),
    Binary(Uuid),
}

pub fn find_for_function(forms: &mut HashMap<Rc<FunctionHead>, FunctionForm>, global_namespace: &mut namespaces::Level, function_head: &Rc<FunctionHead>, implementation: &FunctionImplementation) {
    if implementation.parameter_locals.is_empty() {
        // TODO We could make a helper function and still use a constant even if we use blocks.
        let has_blocks = implementation.expression_tree.values.values().any(|op| matches!(op, ExpressionOperation::Block));
        if !has_blocks {
            global_namespace.insert_name(function_head.function_id, function_head.declared_representation.name.as_str());
            forms.insert(Rc::clone(&function_head), FunctionForm::Constant(function_head.function_id));
            return
        }
    }

    global_namespace.insert_name(function_head.function_id, function_head.declared_representation.name.as_str());
    forms.insert(Rc::clone(&function_head), FunctionForm::FunctionCall(function_head.function_id));
}
