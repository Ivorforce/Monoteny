use std::rc::Rc;
use crate::program::functions::FunctionPointer;
use crate::program::traits::TraitResolution;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct MonomorphicFunction {
    pub pointer: Rc<FunctionPointer>,
    pub resolution: Box<TraitResolution>,
}
