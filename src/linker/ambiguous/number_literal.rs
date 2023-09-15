use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::linker::ambiguous::LinkerAmbiguity;
use crate::linker::imperative::ImperativeLinker;
use crate::linker::LinkError;
use crate::program::calls::FunctionBinding;
use crate::program::computation_tree::{ExpressionID, ExpressionOperation};
use crate::program::traits::{RequirementsFulfillment, TraitGraph};
use crate::program::types::{TypeProto, TypeUnit};

pub struct AmbiguousNumberLiteral {
    pub expression_id: ExpressionID,
    pub value: String,
    pub traits: TraitGraph,
    pub is_float: bool,
}

impl Display for AmbiguousNumberLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ambiguous number literal type: '{}'", self.value)
    }
}

impl LinkerAmbiguity for AmbiguousNumberLiteral {
    fn attempt_to_resolve(&mut self, linker: &mut ImperativeLinker) -> Result<bool, LinkError> {
        let type_ = linker.types.resolve_binding_alias(&self.expression_id)?;
        if TypeProto::contains_generics([&type_].into_iter()) {
            return Ok(false)  // Yet ambiguous
        }

        let literal_expression_id = linker.register_new_expression(vec![]);
        linker.expressions.operations.insert(
            literal_expression_id.clone(),
            ExpressionOperation::StringLiteral(self.value.clone())
        );
        linker.types.bind(literal_expression_id.clone(), TypeProto::unit(TypeUnit::Struct(Rc::clone(&linker.runtime.builtins.core.traits.String))).as_ref())?;

        let trait_ = Rc::clone(if self.is_float { &linker.runtime.builtins.core.traits.ConstructableByFloatLiteral } else { &linker.runtime.builtins.core.traits.ConstructableByIntLiteral });
        let requirement = trait_.create_generic_binding(vec![("self", type_.clone())]);
        let (conformance_tail, conformance) = self.traits.satisfy_requirement(&requirement, &linker.types)?;
        let parse_function = &conformance.function_mapping[
            if self.is_float { &linker.runtime.builtins.core.traits.parse_float_literal_function.target }
            else { &linker.runtime.builtins.core.traits.parse_int_literal_function.target }
        ];

        linker.expressions.arguments.insert(self.expression_id.clone(), vec![literal_expression_id]);
        linker.expressions.operations.insert(
            self.expression_id.clone(),
            ExpressionOperation::FunctionCall(Rc::new(FunctionBinding {
                function: Rc::clone(parse_function),
                requirements_fulfillment: Box::new(RequirementsFulfillment {
                    conformance: HashMap::from([(requirement, (conformance_tail, conformance))]),
                    generic_mapping: HashMap::from([(trait_.generics["self"], type_.clone())])
                } )
            }))
        );
        linker.types.bind(self.expression_id.clone(), type_.as_ref())?;

        Ok(true)
    }
}
