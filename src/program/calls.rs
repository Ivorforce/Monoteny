use std::rc::Rc;
use crate::program::functions::FunctionPointer;
use crate::program::traits::RequirementsFulfillment;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct FunctionBinding {
    pub pointer: Rc<FunctionPointer>,
    pub requirements_fulfillment: Box<RequirementsFulfillment>,
}
