use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Range;
use std::rc::Rc;

use crate::error::{ErrInRange, RResult};
use crate::resolver::ambiguous::{AmbiguityResult, ResolverAmbiguity};
use crate::resolver::imperative::ImperativeResolver;
use crate::program::calls::FunctionBinding;
use crate::program::expression_tree::{ExpressionID, ExpressionOperation};
use crate::program::functions::FunctionHead;
use crate::program::traits::{RequirementsFulfillment, Trait, TraitGraph};

pub struct AmbiguousAbstractCall {
    pub expression_id: ExpressionID,
    pub arguments: Vec<ExpressionID>,
    pub traits: TraitGraph,

    pub range: Range<usize>,

    pub trait_: Rc<Trait>,
    pub abstract_function: Rc<FunctionHead>,
}

impl Display for AmbiguousAbstractCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ambiguous abstract function call.")
    }
}

impl ResolverAmbiguity for AmbiguousAbstractCall {
    fn attempt_to_resolve(&mut self, resolver: &mut ImperativeResolver) -> RResult<AmbiguityResult<()>> {
        let type_ = resolver.types.resolve_binding_alias(&self.expression_id)?;

        let requirement = self.trait_.create_generic_binding(vec![("Self", type_.clone())]);
        let trait_conformance = self.traits.satisfy_requirement(&requirement, &resolver.types)
            .err_in_range(&self.range)?;
        Ok(match trait_conformance {
            AmbiguityResult::Ambiguous => {
                AmbiguityResult::Ambiguous
            }
            AmbiguityResult::Ok(trait_conformance) => {
                let used_function = &trait_conformance.conformance.function_mapping[&self.abstract_function];

                resolver.expression_tree.values.insert(
                    self.expression_id.clone(),
                    ExpressionOperation::FunctionCall(Rc::new(FunctionBinding {
                        function: Rc::clone(used_function),
                        requirements_fulfillment: Rc::new(RequirementsFulfillment {
                            conformance: HashMap::from([(requirement, trait_conformance)]),
                            generic_mapping: HashMap::from([(Rc::clone(&self.trait_.generics["Self"]), type_.clone())])
                        }),
                    }))
                );
                resolver.types.bind(self.expression_id.clone(), type_.as_ref())
                    .err_in_range(&self.range)?;

                AmbiguityResult::Ok(())
            }
        })
    }

    fn get_position(&self) -> Range<usize> {
        self.range.clone()
    }
}
