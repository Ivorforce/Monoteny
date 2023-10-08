use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use uuid::Uuid;
use crate::program::computation_tree::ExpressionOperation;
use crate::program::functions::{FunctionHead, FunctionPointer};
use crate::program::global::FunctionImplementation;
use crate::program::types::TypeProto;
use crate::transpiler::namespaces;

#[derive(Clone)]
pub struct Representations {
    pub builtin_functions: HashSet<Rc<FunctionHead>>,
    pub function_representations: HashMap<Rc<FunctionHead>, FunctionForm>,
    pub type_ids: HashMap<Box<TypeProto>, Uuid>,
}

impl Representations {
    pub fn new() -> Representations {
        Representations {
            builtin_functions: Default::default(),
            function_representations: Default::default(),
            type_ids: Default::default(),
        }
    }
}

// The IDs are attached per object because theoretically it's possible for a representation to use
//  0 names (direct keyword use) or 2 (using multiple keywords). They just 'happen' to all use one.
#[derive(PartialEq, Eq, Clone)]
pub enum FunctionForm {
    CallAsFunction,
    Constant(Uuid),
    FunctionCall(Uuid),
    SetMemberField(Uuid),
    GetMemberField(Uuid),
    MemberCall(Uuid),
    Unary(Uuid),
    Binary(Uuid),
}

pub fn find_for_function(function_representations: &mut HashMap<Rc<FunctionHead>, FunctionForm>, global_namespace: &mut namespaces::Level, implementation: &Box<FunctionImplementation>, pointer: &Rc<FunctionPointer>) {
    if implementation.parameter_locals.is_empty() {
        // TODO We could make a helper function and still use a constant even if we use blocks.
        let has_blocks = implementation.expression_forest.operations.values().any(|op| matches!(op, ExpressionOperation::Block));
        if !has_blocks {
            global_namespace.insert_name(implementation.head.function_id, pointer.name.as_str());
            function_representations.insert(Rc::clone(&implementation.head), FunctionForm::Constant(implementation.head.function_id));
            return
        }
    }

    global_namespace.insert_name(implementation.head.function_id, pointer.name.as_str());
    function_representations.insert(Rc::clone(&implementation.head), FunctionForm::FunctionCall(implementation.head.function_id));
}
