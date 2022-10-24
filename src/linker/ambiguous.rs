use std::collections::HashSet;
use std::rc::Rc;
use itertools::{Itertools, zip_eq};
use uuid::Uuid;
use crate::linker::LinkError;
use crate::program::allocation::{ObjectReference, Reference};
use crate::program::computation_tree::{ExpressionForest, ExpressionID, ExpressionOperation};
use crate::program::functions::FunctionPointer;
use crate::program::generics::TypeForest;
use crate::program::primitives;
use crate::program::primitives::Value;
use crate::program::traits::{TraitBinding, TraitConformanceScope};
use crate::program::types::{TypeProto, TypeUnit};

pub trait LinkerAmbiguity {
    fn attempt_to_resolve(&mut self, expressions: &mut ExpressionForest) -> Result<bool, LinkError>;
}

pub struct AmbiguousNumberPrimitive {
    pub expression_id: ExpressionID,
    pub value: String,
    pub candidates: HashSet<primitives::Type>
}

impl LinkerAmbiguity for AmbiguousNumberPrimitive {
    fn attempt_to_resolve(&mut self, expressions: &mut ExpressionForest) -> Result<bool, LinkError> {
        match expressions.type_forest.get_unit(&self.expression_id) {
            None => Ok(false),  // Not done yet
            Some(TypeUnit::Primitive(primitive_type)) => {
                if !self.candidates.contains(primitive_type) {
                    return Err(LinkError::LinkError { msg: format!("Cannot convert number literal {} to expected primitive type {:?}", &self.value, primitive_type) })
                }

                match primitive_type.parse_value(&self.value) {
                    None => return Err(LinkError::LinkError { msg: format!("Failed to parse {:?} from number primitive {}", primitive_type, &self.value) }),
                    Some(value) => expressions.operations.insert(self.expression_id, ExpressionOperation::Primitive(value)),
                };

                Ok(true)
            }
            unit => {
                Err(LinkError::LinkError { msg: format!("Cannot convert number literal {} to expected type {:?}", self.value, unit) })
            }
        }
    }
}

pub struct AmbiguousFunctionCandidate {
    pub function: Rc<FunctionPointer>,
    pub param_types: Vec<Box<TypeProto>>,
    pub return_type: Box<TypeProto>,
}

pub struct AmbiguousFunctionCall {
    pub expression_id: ExpressionID,
    pub function_name: String,
    pub seed: Uuid,
    pub arguments: Vec<ExpressionID>,
    pub trait_conformance_declarations: TraitConformanceScope,

    pub candidates: Vec<Box<AmbiguousFunctionCandidate>>,
    pub failed_candidates: Vec<(Box<AmbiguousFunctionCandidate>, LinkError)>,
}

impl AmbiguousFunctionCall {
    fn attempt_with_candidate(&self, types: &mut TypeForest, candidate: &AmbiguousFunctionCandidate) -> Result<Box<TraitBinding>, LinkError> {
        let fun = &candidate.function;
        let param_types = &candidate.param_types;

        for (arg, param) in zip_eq(
            self.arguments.iter(),
            param_types.iter().map(|x| x.as_ref())
        ) {
            types.bind(arg.clone(), param)?;
        }
        types.bind(self.expression_id.clone(), &candidate.return_type)?;

        let binding = self.trait_conformance_declarations
            .satisfy_requirements(&fun.machine_interface.requirements, &self.seed, &types)?;

        Ok(binding)
    }
}

impl LinkerAmbiguity for AmbiguousFunctionCall {
    fn attempt_to_resolve(&mut self, expressions: &mut ExpressionForest) -> Result<bool, LinkError> {
        let mut is_ambiguous = false;
        for candidate in self.candidates.drain(..).collect_vec() {
            let mut types_copy = expressions.type_forest.clone();
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
            let binding = self.attempt_with_candidate(&mut expressions.type_forest, &candidate)?;

            let argument_targets: Vec<Rc<ObjectReference>> = candidate.function.human_interface.parameter_names.iter()
                .map(|x| Rc::clone(&x.1))
                .collect();

            expressions.operations.insert(self.expression_id, ExpressionOperation::FunctionCall {
                function: candidate.function,
                argument_targets,
                binding
            });

            // We're done!
            return Ok(true)
        }

        // TODO We should probably output the locations of candidates.

        let argument_types = self.arguments.iter().map(|t|
            expressions.type_forest.prototype_binding_alias(t)
        ).collect_vec();

        if self.failed_candidates.len() == 1 {
            // TODO How so?
            let (candidate, err) = self.failed_candidates.iter().next().unwrap();

            panic!("function {:?} could not be resolved. Candidate failed type / requirements test: {:?}", &candidate.function.human_interface, err)
        } else {
            // TODO Print types of arguments too, for context.
            panic!("function {} could not be resolved. {} candidates failed type / requirements test: {:?}", self.function_name, self.failed_candidates.len(), &argument_types)
        }
    }
}
