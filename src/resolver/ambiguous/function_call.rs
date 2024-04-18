use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Range;
use std::rc::Rc;

use itertools::{Itertools, zip_eq};

use crate::error::{format_errors, RResult, RuntimeError};
use crate::resolver::ambiguous::{AmbiguityResult, ResolverAmbiguity};
use crate::resolver::imperative::ImperativeResolver;
use crate::program::calls::FunctionBinding;
use crate::program::debug::MockFunctionInterface;
use crate::program::expression_tree::{ExpressionID, ExpressionOperation};
use crate::program::function_object::FunctionRepresentation;
use crate::program::functions::{FunctionHead, ParameterKey};
use crate::program::generics::TypeForest;
use crate::program::traits::{RequirementsFulfillment, Trait, TraitBinding, TraitGraph};
use crate::program::types::TypeProto;

pub struct AmbiguousFunctionCandidate {
    pub function: Rc<FunctionHead>,
    pub generic_map: HashMap<Rc<Trait>, Rc<TypeProto>>,
    // All these are seeded already
    pub param_types: Vec<Rc<TypeProto>>,
    pub return_type: Rc<TypeProto>,
    pub requirements: Vec<Rc<TraitBinding>>,
}

pub struct AmbiguousFunctionCall {
    pub expression_id: ExpressionID,
    pub representation: FunctionRepresentation,
    pub arguments: Vec<ExpressionID>,
    pub traits: TraitGraph,

    pub range: Range<usize>,

    pub candidates: Vec<Box<AmbiguousFunctionCandidate>>,
    pub failed_candidates: Vec<(Box<AmbiguousFunctionCandidate>, Vec<RuntimeError>)>,
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
            match self.traits.satisfy_requirement(&requirement.mapping_types(&|type_| type_.replacing_structs(&candidate.generic_map)), &types)? {
                AmbiguityResult::Ok(trait_conformance) => {
                    conformance.insert(requirement, trait_conformance);
                }
                AmbiguityResult::Ambiguous => return Ok(AmbiguityResult::Ambiguous),
            }
        }

        Ok(AmbiguityResult::Ok(Rc::new(RequirementsFulfillment { generic_mapping: candidate.generic_map.clone(), conformance })))
    }
}

impl Display for AmbiguousFunctionCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ambiguous function call ({} candidates): {}", self.candidates.len(), self.representation.name)
    }
}

impl ResolverAmbiguity for AmbiguousFunctionCall {
    fn attempt_to_resolve(&mut self, resolver: &mut ImperativeResolver) -> RResult<AmbiguityResult<()>> {
        let mut is_ambiguous = false;
        for candidate in self.candidates.drain(..).collect_vec() {
            let mut types_copy = resolver.types.clone();
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
            // TODO We can just assign resolver.types to the candidate's result; it was literally just copied.
            match self.attempt_with_candidate(&mut resolver.types, &candidate)? {
                AmbiguityResult::Ok(resolution) => {
                    resolver.expression_tree.values.insert(self.expression_id, ExpressionOperation::FunctionCall(Rc::new(FunctionBinding {
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

        match &self.failed_candidates[..] {
            [] => panic!(),
            [(candidate, err)] => {
                // TODO How so?
                Err(RuntimeError::new(format!("function {:?} could not be resolved. Candidate failed type / requirements test with error:\n{}", &candidate.function, format_errors(err))))
            }
            cs => {
                let signature = MockFunctionInterface {
                    representation: self.representation.clone(),
                    argument_keys: self.arguments.iter().map(|a| ParameterKey::Positional).collect_vec(),
                    arguments: self.arguments.clone(),
                    types: &resolver.types,
                };
                Err(RuntimeError::new(format!("function {} could not be resolved. {} candidates failed type / requirements test.", signature, cs.len())))
            }
        }
    }
}
