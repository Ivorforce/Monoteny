use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::rc::Rc;
use itertools::Itertools;
use uuid::Uuid;
use crate::program::allocation::ObjectReference;
use crate::program::calls::FunctionBinding;
use crate::program::computation_tree::{ExpressionForest, ExpressionOperation, Statement};
use crate::program::functions::{FunctionHead, FunctionType, FunctionInterface, Parameter};
use crate::program::generics::TypeForest;
use crate::program::global::FunctionImplementation;
use crate::program::traits::{RequirementsAssumption, RequirementsFulfillment, TraitConformance};
use crate::program::types::TypeProto;


pub struct Monomorphizer {
    pub new_encountered_calls: Vec<Rc<FunctionBinding>>,
    pub encountered_calls: HashSet<Rc<FunctionBinding>>,
    // Not every call that is mapped is actually encountered.
    // The primary reason is that for polymorphic function calls, EVERY function in the
    //  fulfillment needs to be mapped upon call, but isn't necessarily actually called.
    pub resolved_call_to_mono_call: HashMap<Rc<FunctionBinding>, Rc<FunctionBinding>>,
}

impl Monomorphizer {
    pub fn new() -> Monomorphizer {
        Monomorphizer {
            new_encountered_calls: Default::default(),
            encountered_calls: Default::default(),
            resolved_call_to_mono_call: Default::default(),
        }
    }

    pub fn monomorphize_function(&mut self, implementation: &FunctionImplementation, function_binding: &Rc<FunctionBinding>, should_monomorphize: &dyn Fn(&Rc<FunctionBinding>) -> bool) -> Rc<FunctionImplementation> {
        // Map types.
        let mut type_forest = implementation.type_forest.clone();

        let generic_replacement_map = &function_binding.requirements_fulfillment.generic_mapping;

        // Change Anys to Generics in the type forest.
        type_forest.bind_any_as_generic(generic_replacement_map).unwrap();

        let mut expression_forest = Box::new(ExpressionForest::new());

        // Map variables.
        // TODO For fully internal variables, it would be enough to set the type to the Any's corresponding Generic,
        //  because those have been bound in the type forest. For variables featured in the interface, however, the
        //  type must be properly resolved. So we might as well map all variables to resolved types.
        let variable_map: HashMap<Rc<ObjectReference>, Rc<ObjectReference>> = implementation.variable_names.keys()
            .map(|v| {
                (Rc::clone(v), map_variable(v, &generic_replacement_map))
            })
            .collect();

        // Map statements. Expressions are mapped elsewhere, so this should be easy.
        let statements = implementation.statements.iter().map(|x| {
            Box::new(match x.as_ref() {
                Statement::VariableAssignment(v, e) => {
                    Statement::VariableAssignment(Rc::clone(&variable_map[v]), e.clone())
                },
                _ => x.as_ref().clone(),
            })
        }).collect_vec();

        let mut function_replacement_map = HashMap::new();
        for conformance in implementation.requirements_assumption.conformance.values() {
            for (abstract_fun, fun_placement) in conformance.function_mapping.iter() {
                let binds = &function_binding.requirements_fulfillment.conformance[&conformance.binding.mapping_types(&|type_| type_.unfreezing_any_to_generics())].function_mapping;
                let replacement = &binds[abstract_fun];
                function_replacement_map.insert(Rc::clone(fun_placement), Rc::clone(replacement));
            }
        }

        // Find function calls in the expression forest
        for (expression_id, operation) in implementation.expression_forest.operations.iter() {
            expression_forest.operations.insert(expression_id.clone(), match operation {
                ExpressionOperation::FunctionCall(call) => {
                    let resolved_call = resolve_call(call, &generic_replacement_map, &function_replacement_map, &type_forest);

                    if self.encountered_calls.insert(Rc::clone(&resolved_call)) {
                        self.new_encountered_calls.push(Rc::clone(&resolved_call));
                    }

                    let mono_call: Rc<FunctionBinding> = if !resolved_call.requirements_fulfillment.is_empty() && should_monomorphize(&resolved_call) {
                        self.monomorphize_call(&resolved_call)
                    }
                    else {
                        resolved_call
                    };

                    ExpressionOperation::FunctionCall(mono_call)
                }
                ExpressionOperation::PairwiseOperations { calls } => {
                    ExpressionOperation::PairwiseOperations {
                        calls: calls.iter()
                            .map(|call| {
                                let resolved_call = resolve_call(call, &generic_replacement_map, &function_replacement_map, &type_forest);

                                if self.encountered_calls.insert(Rc::clone(&resolved_call)) {
                                    self.new_encountered_calls.push(Rc::clone(&resolved_call));
                                }

                                let mono_call: Rc<FunctionBinding> = if !resolved_call.requirements_fulfillment.is_empty() && should_monomorphize(&resolved_call) {
                                    self.monomorphize_call(&resolved_call)
                                }
                                else {
                                    resolved_call
                                };

                                mono_call
                            }).collect_vec()
                    }
                }
                ExpressionOperation::VariableLookup(v) => {
                    // If we cannot find a replacement, it's a static variable. Unless we have a bug.
                    ExpressionOperation::VariableLookup(Rc::clone(variable_map.get(v).unwrap_or(v)))
                }
                ExpressionOperation::ArrayLiteral => ExpressionOperation::ArrayLiteral,
                ExpressionOperation::StringLiteral(s) => ExpressionOperation::StringLiteral(s.clone()),
            });
        }
        expression_forest.arguments = implementation.expression_forest.arguments.clone();

        let monomorphized_binding = &self.resolved_call_to_mono_call.get(function_binding).unwrap_or(function_binding);
        Rc::new(FunctionImplementation {
            implementation_id: Uuid::new_v4(),
            head: Rc::clone(&monomorphized_binding.function),  // Re-use premapped pointer
            decorators: implementation.decorators.clone(),
            // TODO This is correct only if all requirements have been fulfilled.
            //  If monomorphize was requested on a partially generic function, we continue to
            //  have some requirements.
            requirements_assumption: Box::new(RequirementsAssumption { conformance: Default::default() }),
            statements,
            expression_forest,
            type_forest,
            parameter_variables: implementation.parameter_variables.iter().map(|x| Rc::clone(&variable_map[x])).collect_vec(),
            variable_names: implementation.variable_names.clone(),  // TODO The variable references change as the variables themselves change type
        })
    }

    fn monomorphize_call(&mut self, resolved_call: &Rc<FunctionBinding>) -> Rc<FunctionBinding> {
        match self.resolved_call_to_mono_call.entry(Rc::clone(&resolved_call)) {
            Entry::Occupied(o) => Rc::clone(o.get()),
            Entry::Vacant(v) => {
                Rc::clone(v.insert(monomorphize_call(&resolved_call)))
            },
        }
    }

    pub fn get_mono_call_to_original_call(&self) -> HashMap<Rc<FunctionHead>, Rc<FunctionHead>> {
        self.resolved_call_to_mono_call.iter()
            .map(|(x, y)| (Rc::clone(&y.function), Rc::clone(&x.function)))
            .collect()
    }
}

pub fn resolve_call(call: &Rc<FunctionBinding>, generic_replacement_map: &HashMap<Uuid, Box<TypeProto>>, function_replacement_map: &HashMap<Rc<FunctionHead>, Rc<FunctionHead>>, type_forest: &TypeForest) -> Rc<FunctionBinding> {
    let mapped_call = function_replacement_map.get(&call.function).unwrap_or(&call.function);

    let mut generic_replacement_map: HashMap<Uuid, Box<TypeProto>> = call.requirements_fulfillment.generic_mapping.iter().map(|(any_id, type_)| {
        (*any_id, type_forest.resolve_type(type_).unwrap().replacing_anys(generic_replacement_map))
    }).collect();

    Rc::new(FunctionBinding {
        function: Rc::clone(mapped_call),
        requirements_fulfillment: Box::new(RequirementsFulfillment {
            conformance: call.requirements_fulfillment.conformance.iter()
                .map(|(requirement, conformance)| {
                    (Rc::clone(requirement), TraitConformance::new(
                        Rc::clone(requirement),
                        conformance.function_mapping.iter()
                            .map(
                                |(abstract_fun, fulfillment_fun)|
                                (Rc::clone(abstract_fun), Rc::clone(function_replacement_map.get(fulfillment_fun).unwrap_or_else(|| fulfillment_fun)))
                            )
                            .collect()
                    ))
                })
                .collect(),
            generic_mapping: generic_replacement_map,
        }),
    })
}

pub fn monomorphize_call(call: &Rc<FunctionBinding>) -> Rc<FunctionBinding> {
    // TODO If we're not fully monomorphized, which might be the case if we're transpiling a generic function, we
    //   - are not yet static
    //   - still require a requirements fulfillment
    Rc::new(FunctionBinding {
        function: Rc::new(FunctionHead {
            function_id: Uuid::new_v4(),
            // We're now a static call!
            function_type: FunctionType::Static,
            interface: Rc::new(map_interface_types(&call.function.interface, &|x| x.replacing_generics(&call.requirements_fulfillment.generic_mapping))),
        }),
        // We are finally empty now!
        requirements_fulfillment: RequirementsFulfillment::empty(),
    })
}

pub fn map_variable(variable: &ObjectReference, type_replacement_map: &HashMap<Uuid, Box<TypeProto>>) -> Rc<ObjectReference> {
    Rc::new(ObjectReference {
        id: variable.id.clone(),
        type_: variable.type_.replacing_anys(type_replacement_map),
        mutability: variable.mutability.clone(),
    })
}

pub fn map_interface_types(interface: &FunctionInterface, map: &dyn Fn(&Box<TypeProto>) -> Box<TypeProto>) -> FunctionInterface{
    FunctionInterface {
        parameters: interface.parameters.iter().map(|x| Parameter {
            external_key: x.external_key.clone(),
            internal_name: x.internal_name.clone(),
            type_: map(&x.type_),
        }).collect(),
        return_type: map(&interface.return_type),
        requirements: interface.requirements.iter().map(|x| x.mapping_types(map)).collect(),
    }
}
