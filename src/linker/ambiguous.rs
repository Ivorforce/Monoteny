use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use itertools::{Itertools, zip_eq};
use uuid::Uuid;
use crate::linker::imperative::ImperativeLinker;
use crate::linker::LinkError;
use crate::program::allocation::{ObjectReference, Reference};
use crate::program::calls::FunctionCall;
use crate::program::computation_tree::{ExpressionForest, ExpressionID, ExpressionOperation};
use crate::program::functions::FunctionPointer;
use crate::program::generics::TypeForest;
use crate::program::primitives;
use crate::program::traits::{TraitResolution, TraitRequirement, TraitConformanceScope, TraitBinding};
use crate::program::types::{TypeProto, TypeUnit};

pub trait LinkerAmbiguity {
    fn attempt_to_resolve(&mut self, expressions: &mut ImperativeLinker) -> Result<bool, LinkError>;
}

pub struct AmbiguousNumberPrimitive {
    pub expression_id: ExpressionID,
    pub value: String,
    pub traits: TraitConformanceScope,
    pub is_float: bool,
}

impl LinkerAmbiguity for AmbiguousNumberPrimitive {
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
                let requirement = Box::new(TraitBinding {
                    generic_to_type: HashMap::from([(*trait_.generics.iter().next().unwrap(), type_.clone())]),
                    trait_,
                });
                let resolution = self.traits.satisfy_requirements(
                    &vec![requirement], &linker.types
                )?;
                let parse_function = &resolution.function_binding[
                    if self.is_float { &linker.builtins.core.traits.parse_float_literal_function } else { &linker.builtins.core.traits.parse_int_literal_function }
                ];

                linker.expressions.arguments.insert(self.expression_id.clone(), vec![literal_expression_id]);
                linker.expressions.operations.insert(
                    self.expression_id.clone(),
                    ExpressionOperation::FunctionCall(FunctionCall { pointer: Rc::clone(parse_function), resolution })
                );
                linker.types.bind(self.expression_id.clone(), type_.as_ref())?;

                Ok(true)
            }
        }
    }
}

pub struct AmbiguousFunctionCandidate {
    pub function: Rc<FunctionPointer>,
    // All these are seeded already
    pub param_types: Vec<Box<TypeProto>>,
    pub return_type: Box<TypeProto>,
    pub requirements: Vec<Box<TraitBinding>>,

}
pub struct AmbiguousFunctionCall {
    pub expression_id: ExpressionID,
    pub function_name: String,
    pub arguments: Vec<ExpressionID>,
    pub trait_conformance_declarations: TraitConformanceScope,

    pub candidates: Vec<Box<AmbiguousFunctionCandidate>>,
    pub failed_candidates: Vec<(Box<AmbiguousFunctionCandidate>, LinkError)>,
}

impl AmbiguousFunctionCall {
    fn attempt_with_candidate(&self, types: &mut TypeForest, candidate: &AmbiguousFunctionCandidate) -> Result<Box<TraitResolution>, LinkError> {
        let param_types = &candidate.param_types;

        for (arg, param) in zip_eq(
            self.arguments.iter(),
            param_types.iter().map(|x| x.as_ref())
        ) {
            types.bind(arg.clone(), param)?;
        }
        types.bind(self.expression_id.clone(), &candidate.return_type)?;

        let binding = self.trait_conformance_declarations
            .satisfy_requirements(
                &candidate.requirements,
                &types
            )?;

        Ok(binding)
    }
}

impl LinkerAmbiguity for AmbiguousFunctionCall {
    fn attempt_to_resolve(&mut self, linker: &mut ImperativeLinker) -> Result<bool, LinkError> {
        let mut is_ambiguous = false;
        for candidate in self.candidates.drain(..).collect_vec() {
            let mut types_copy = linker.types.clone();
            let result = self.attempt_with_candidate(&mut types_copy, &candidate);

            match result {
                Ok(_) => self.candidates.push(candidate),
                Err(LinkError::Ambiguous) => {
                    self.candidates.push(candidate);
                    is_ambiguous = true;
                }
                Err(err) => {
                    self.failed_candidates.push((candidate, err));
                }
            }
        }

        // Still ambiguous!
        if is_ambiguous || self.candidates.len() > 1 {
            return Ok(false)
        }

        if self.candidates.len() == 1 {
            let candidate = self.candidates.drain(..).next().unwrap();
            // TODO We can just assign linker.types to the candidate's result; it was łiterally just copied.
            let resolution = self.attempt_with_candidate(&mut linker.types, &candidate)?;

            linker.expressions.operations.insert(self.expression_id, ExpressionOperation::FunctionCall(FunctionCall {
                pointer: candidate.function,
                resolution
            }));

            // We're done!
            return Ok(true)
        }

        // TODO We should probably output the locations of candidates.

        let argument_types = self.arguments.iter().map(|t|
            linker.types.prototype_binding_alias(t)
        ).collect_vec();

        if self.failed_candidates.len() == 1 {
            // TODO How so?
            let (candidate, err) = self.failed_candidates.iter().next().unwrap();

            Err(LinkError::LinkError { msg: format!("function {:?} could not be resolved. Candidate failed type / requirements test: {}", &candidate.function, err) })
        } else {
            // TODO Print types of arguments too, for context.
            Err(LinkError::LinkError { msg: format!("function {} could not be resolved. {} candidates failed type / requirements test: {:?}", self.function_name, self.failed_candidates.len(), &argument_types) })
        }
    }
}
