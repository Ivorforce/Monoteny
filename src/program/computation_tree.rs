use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use guard::guard;
use uuid::Uuid;
use crate::parser::abstract_syntax::Function;
use crate::program::types::{Mutability, ParameterKey, Type, Variable};

use crate::program::builtins::TenLangBuiltins;
use crate::program::functions::{FunctionPointer, HumanFunctionInterface, MachineFunctionInterface};
use crate::program::primitives;
use crate::program::traits::{Trait, TraitBinding, TraitConformanceDeclaration, TraitConformanceRequirement};

// ================================ Global ==============================

pub struct Program {
    pub functions: HashSet<Rc<FunctionImplementation>>,
    pub traits: HashSet<Rc<Trait>>,
    pub main_function: Option<Rc<FunctionImplementation>>,
}

pub struct FunctionImplementation {
    pub implementation_id: Uuid,
    pub function_id: Uuid,

    pub human_interface: Rc<HumanFunctionInterface>,
    pub machine_interface: Rc<MachineFunctionInterface>,

    pub conformance_delegations: HashMap<Rc<TraitConformanceRequirement>, Rc<TraitConformanceDeclaration>>,

    pub statements: Vec<Box<Statement>>,
    pub variable_names: HashMap<Rc<Variable>, String>,
}

// ================================ Code ==============================

pub enum Statement {
    VariableAssignment(Rc<Variable>, Box<Expression>),
    Expression(Box<Expression>),
    Return(Option<Box<Expression>>),
}

pub struct Expression {
    pub result_type: Option<Box<Type>>,
    pub operation: Box<ExpressionOperation>,
}

pub enum ExpressionOperation {
    Primitive(primitives::Value),
    FunctionCall { function: Rc<FunctionPointer>, arguments: HashMap<Rc<Variable>, Box<Expression>>, binding: Box<TraitBinding> },
    PairwiseOperations { arguments: Vec<Box<Expression>>, functions: Vec<Rc<HumanFunctionInterface>> },
    MemberLookup(Box<Expression>, String),
    VariableLookup(Rc<Variable>),
    StringLiteral(String),
    ArrayLiteral(Vec<Box<Expression>>),
}

impl PartialEq for FunctionImplementation {
    fn eq(&self, other: &Self) -> bool {
        self.implementation_id == other.implementation_id
    }
}

impl Eq for FunctionImplementation {}

impl Hash for FunctionImplementation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.implementation_id.hash(state);
    }
}
