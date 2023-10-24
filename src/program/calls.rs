use std::rc::Rc;
use crate::program::functions::FunctionHead;
use crate::program::generics::TypeForest;
use crate::program::traits::{RequirementsFulfillment, TraitConformance, TraitConformanceWithTail};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
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

pub fn resolve_binding(binding: &FunctionBinding, type_forest: &TypeForest) -> Rc<FunctionBinding> {
    Rc::new(FunctionBinding {
        function: Rc::clone(&binding.function),
        requirements_fulfillment: resolve_fulfillment(&binding.requirements_fulfillment, type_forest),
    })
}

pub fn resolve_fulfillment(fulfillment: &RequirementsFulfillment, type_forest: &TypeForest) -> Rc<RequirementsFulfillment> {
    Rc::new(RequirementsFulfillment {
        conformance: fulfillment.conformance.iter().map(|(b, f)| {
            let binding = b.mapping_types(&|t| type_forest.resolve_type(t).unwrap());
            (Rc::clone(&binding), Rc::new(TraitConformanceWithTail {
                conformance: Rc::new(TraitConformance {
                    binding,
                    function_mapping: f.conformance.function_mapping.clone(),
                }),
                tail: resolve_fulfillment(&f.tail, type_forest),
            }))
        }).collect(),
        generic_mapping: fulfillment.generic_mapping.iter()
            .map(|(generic, type_)| (Rc::clone(generic), type_forest.resolve_type(type_).unwrap()))
            .collect(),
    })
}
