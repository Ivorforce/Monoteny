use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use guard::guard;
use uuid::Uuid;
use crate::parser::abstract_syntax::Function;
use crate::program::allocation::{Mutability, Variable};
use crate::program::types::TypeProto;

use crate::program::builtins::Builtins;
use crate::program::functions::{FunctionPointer, HumanFunctionInterface, MachineFunctionInterface, ParameterKey};
use crate::program::generics::{GenericAlias, TypeForest};
use crate::program::primitives;
use crate::program::traits::{Trait, TraitBinding, TraitConformanceDeclaration, TraitConformanceRequirement};

pub type ExpressionID = GenericAlias;

pub enum Statement {
    VariableAssignment(Rc<Variable>, ExpressionID),
    Expression(ExpressionID),
    Return(Option<ExpressionID>),
}

pub enum ExpressionOperation {
    Primitive(primitives::Value),
    FunctionCall { function: Rc<FunctionPointer>, argument_targets: Vec<Rc<Variable>>, binding: Box<TraitBinding> },
    PairwiseOperations { functions: Vec<Rc<HumanFunctionInterface>> },
    MemberLookup(String),
    VariableLookup(Rc<Variable>),
    StringLiteral(String),
    ArrayLiteral,
}

pub struct ExpressionForest {
    /// Expressions' return types can be looked up under the expressions' IDs.
    pub type_forest: Box<TypeForest>,

    /// Will be set for every expression ID
    pub arguments: HashMap<ExpressionID, Vec<ExpressionID>>,
    /// Might not be set for a while
    pub operations: HashMap<ExpressionID, ExpressionOperation>,
}

impl ExpressionForest {
    pub fn new() -> ExpressionForest {
        ExpressionForest {
            type_forest: Box::new(TypeForest::new() ),
            operations: HashMap::new(),
            arguments: HashMap::new(),
        }
    }

    pub fn register_new_expression(&mut self, arguments: Vec<ExpressionID>) -> ExpressionID {
        let id = ExpressionID::new_v4();

        self.type_forest.register(id);
        self.arguments.insert(id, arguments);

        id
    }
}
