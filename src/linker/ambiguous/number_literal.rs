use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::linker::ambiguous::LinkerAmbiguity;
use crate::linker::imperative::ImperativeLinker;
use crate::linker::LinkError;
use crate::program::calls::MonomorphicFunction;
use crate::program::computation_tree::{ExpressionID, ExpressionOperation};
use crate::program::traits::{TraitGraph, TraitResolution};
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
        match linker.types.resolve_binding_alias(&self.expression_id) {
            Err(_) => Ok(false),  // Not done yet
            Ok(type_) => {
                let literal_expression_id = linker.register_new_expression(vec![]);
                linker.expressions.operations.insert(
                    literal_expression_id.clone(),
                    ExpressionOperation::StringLiteral(self.value.clone())
                );
                linker.types.bind(literal_expression_id.clone(), TypeProto::unit(TypeUnit::Struct(Rc::clone(&linker.builtins.core.traits.String))).as_ref())?;

                let trait_ = Rc::clone(if self.is_float { &linker.builtins.core.traits.ConstructableByFloatLiteral } else { &linker.builtins.core.traits.ConstructableByIntLiteral });
                let requirement = trait_.create_generic_binding(vec![(&"self".into(), type_.clone())]);
                let function_resolution = self.traits.satisfy_requirement(&requirement, &linker.types)?;
                let parse_function = &function_resolution[
                    if self.is_float { &linker.builtins.core.traits.parse_float_literal_function }
                    else { &linker.builtins.core.traits.parse_int_literal_function }
                    ];

                linker.expressions.arguments.insert(self.expression_id.clone(), vec![literal_expression_id]);
                linker.expressions.operations.insert(
                    self.expression_id.clone(),
                    ExpressionOperation::FunctionCall(Rc::new(MonomorphicFunction {
                        pointer: Rc::clone(parse_function),
                        resolution: Box::new(TraitResolution { conformance: HashMap::from([(requirement, function_resolution)]) } )
                    }))
                );
                linker.types.bind(self.expression_id.clone(), type_.as_ref())?;

                Ok(true)
            }
        }
    }
}
