use std::rc::Rc;
use crate::program::functions::FunctionHead;
use crate::program::traits::RequirementsFulfillment;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct FunctionBinding {
    pub function: Rc<FunctionHead>,
    pub requirements_fulfillment: Box<RequirementsFulfillment>,
}
