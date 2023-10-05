use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Range;
use std::rc::Rc;
use itertools::{Itertools, zip_eq};
use uuid::Uuid;
use crate::error::{RResult, RuntimeError};
use crate::linker::ambiguous::{AmbiguityResult, LinkerAmbiguity};
use crate::linker::imperative::ImperativeLinker;
use crate::program::calls::FunctionBinding;
use crate::program::computation_tree::{ExpressionID, ExpressionOperation};
use crate::program::functions::FunctionHead;
use crate::program::generics::TypeForest;
use crate::program::traits::{RequirementsFulfillment, TraitBinding, TraitConformanceWithTail, TraitGraph};
use crate::program::types::{TypeProto, TypeUnit};

pub struct AmbiguousFunctionCandidate {
    pub function: Rc<FunctionHead>,
    // All these are seeded already
    pub param_types: Vec<Box<TypeProto>>,
    pub return_type: Box<TypeProto>,
    pub requirements: Vec<Rc<TraitBinding>>,
}

pub struct AmbiguousFunctionCall {
    pub seed: Uuid,
    pub expression_id: ExpressionID,
    pub function_name: String,
    pub arguments: Vec<ExpressionID>,
    pub traits: TraitGraph,

    pub range: Range<usize>,

    pub candidates: Vec<Box<AmbiguousFunctionCandidate>>,
    pub failed_candidates: Vec<(Box<AmbiguousFunctionCandidate>, RuntimeError)>,
}

impl AmbiguousFunctionCall {
    fn attempt_with_candidate(&mut self, types: &mut TypeForest, candidate: &AmbiguousFunctionCandidate) -> RResult<AmbiguityResult<Rc<RequirementsFulfillment>>> {
        let param_types = &candidate.param_types;

        for (arg, param) in zip_eq(
            self.arguments.iter(),
            param_types.iter().map(|x| x.as_ref())
        ) {
            types.bind(arg.clone(), param)?;
        }
        types.bind(self.expression_id.clone(), &candidate.return_type)?;

        // Currently, our resolution is just pointing to generics. But that's good enough!
        let mut conformance = HashMap::new();
        // TODO We should only use deep requirements once we actually use this candidate.
        //  The deep ones are guaranteed to exist if the original requirements can be satisfied.
        for requirement in self.traits.gather_deep_requirements(candidate.requirements.iter().cloned()) {
            match self.traits.satisfy_requirement(&requirement, &types)? {
                AmbiguityResult::Ok(trait_conformance) => {
                    conformance.insert(requirement.mapping_types(&|x| x.seeding_generics(&self.seed)), trait_conformance);
                }
                AmbiguityResult::Ambiguous => return Ok(AmbiguityResult::Ambiguous),
            }
        }

        let generic_mapping: HashMap<_, _> = candidate.function.interface.collect_generics().iter().map(|id| {
            (*id, TypeProto::unit(TypeUnit::Generic(TypeProto::bitxor(id, &self.seed))))
        }).collect();

        Ok(AmbiguityResult::Ok(Rc::new(RequirementsFulfillment { generic_mapping, conformance })))
    }
}

impl Display for AmbiguousFunctionCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ambiguous function call ({} candidates): {}", self.candidates.len(), self.function_name)
    }
}

impl LinkerAmbiguity for AmbiguousFunctionCall {
    fn attempt_to_resolve(&mut self, linker: &mut ImperativeLinker) -> RResult<AmbiguityResult<()>> {
        let mut is_ambiguous = false;
        for candidate in self.candidates.drain(..).collect_vec() {
            let mut types_copy = linker.types.clone();
            let result = self.attempt_with_candidate(&mut types_copy, &candidate);

            match result {
                Ok(AmbiguityResult::Ok(_)) => self.candidates.push(candidate),
                Ok(AmbiguityResult::Ambiguous) => {
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
            return Ok(AmbiguityResult::Ambiguous)
        }

        if self.candidates.len() == 1 {
            let candidate = self.candidates.drain(..).next().unwrap();
            // TODO We can just assign linker.types to the candidate's result; it was literally just copied.
            match self.attempt_with_candidate(&mut linker.types, &candidate)? {
                AmbiguityResult::Ok(resolution) => {
                    linker.expressions.operations.insert(self.expression_id, ExpressionOperation::FunctionCall(Rc::new(FunctionBinding {
                        function: Rc::clone(&candidate.function),
                        requirements_fulfillment: resolution
                    })));

                    // We're done!
                    return Ok(AmbiguityResult::Ok(()))
                }
                AmbiguityResult::Ambiguous => {
                    return Ok(AmbiguityResult::Ambiguous)
                }
            }
        }

        // TODO We should probably output the locations of candidates.

        let argument_types = self.arguments.iter().map(|t|
            linker.types.prototype_binding_alias(t)
        ).collect_vec();

        match &self.failed_candidates[..] {
            [] => panic!(),
            [c] => {
                // TODO How so?
                let (candidate, err) = self.failed_candidates.iter().next().unwrap();

                Err(RuntimeError::new(format!("function {:?} could not be resolved. Candidate failed type / requirements test: {}", &candidate.function, err)))
            }
            cs => {
                // TODO Print types of arguments too, for context.
                Err(RuntimeError::new(format!("function {} could not be resolved. {} candidates failed type / requirements test: {:?}", self.function_name, self.failed_candidates.len(), &argument_types)))
            }
        }
    }
}
