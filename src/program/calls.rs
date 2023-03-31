use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use uuid::Uuid;
use crate::program::functions::FunctionPointer;
use crate::program::traits::{TraitResolution, TraitRequirement};
use crate::program::types::TypeProto;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct FunctionCall {
    pub pointer: Rc<FunctionPointer>,
    pub resolution: Box<TraitResolution>,
}
