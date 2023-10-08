use std::rc::Rc;
use crate::program::functions::FunctionHead;
use crate::program::traits::RequirementsFulfillment;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct FunctionBinding {
    pub function: Rc<FunctionHead>,
    pub requirements_fulfillment: Rc<RequirementsFulfillment>,
}

impl FunctionBinding {
    pub fn pure(function: Rc<FunctionHead>) -> Rc<FunctionBinding> {
        Rc::new(FunctionBinding {
            function,
            requirements_fulfillment: RequirementsFulfillment::empty(),
        })
    }
}
