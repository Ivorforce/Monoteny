use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use guard::guard;
use crate::linker::ambiguous::LinkerAmbiguity;
use crate::linker::imperative::ImperativeLinker;
use crate::linker::LinkError;
use crate::program::calls::FunctionBinding;
use crate::program::computation_tree::{ExpressionID, ExpressionOperation};
use crate::program::functions::FunctionHead;
use crate::program::traits::{RequirementsFulfillment, Trait, TraitConformanceWithTail, TraitGraph};
use crate::program::types::{TypeProto, TypeUnit};

pub struct AmbiguousAbstractCall {
    pub expression_id: ExpressionID,
    pub arguments: Vec<ExpressionID>,
    pub traits: TraitGraph,

    pub interface: Rc<Trait>,
    pub abstract_function: Rc<FunctionHead>,
}

impl Display for AmbiguousAbstractCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ambiguous abstract function call.")
    }
}

impl LinkerAmbiguity for AmbiguousAbstractCall {
    fn attempt_to_resolve(&mut self, linker: &mut ImperativeLinker) -> Result<bool, LinkError> {
        let type_ = linker.types.resolve_binding_alias(&self.expression_id)?;

        let requirement = self.interface.create_generic_binding(vec![("Self", type_.clone())]);
        let trait_conformance = self.traits.satisfy_requirement(&requirement, &linker.types);
        match trait_conformance {
            Err(LinkError::Ambiguous) => {
                Ok(false)
            }
            Err(err) => Err(err),
            Ok(trait_conformance) => {
                let used_function = &trait_conformance.conformance.function_mapping[&self.abstract_function];

                linker.expressions.operations.insert(
                    self.expression_id.clone(),
                    ExpressionOperation::FunctionCall(Rc::new(FunctionBinding {
                        function: Rc::clone(used_function),
                        requirements_fulfillment: Rc::new(RequirementsFulfillment {
                            conformance: HashMap::from([(requirement, trait_conformance)]),
                            generic_mapping: HashMap::from([(self.interface.generics["Self"], type_.clone())])
                        }),
                    }))
                );
                linker.types.bind(self.expression_id.clone(), type_.as_ref())?;

                Ok(true)
            }
        }
    }
}
