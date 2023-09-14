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
use crate::program::traits::{RequirementsAssumption, RequirementsFulfillment, TraitBinding, TraitConformance};
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

    pub fn monomorphize_function(&mut self, implementation: &mut Box<FunctionImplementation>, function_binding: &Rc<FunctionBinding>, should_monomorphize: &dyn Fn(&Rc<FunctionBinding>) -> bool) {
        // Map types.
        let generic_replacement_map = &function_binding.requirements_fulfillment.generic_mapping;

        // Change Anys to Generics in the type forest.
        implementation.type_forest.bind_any_as_generic(generic_replacement_map).unwrap();

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
        implementation.statements.iter_mut().map(|x| {
            Box::new(match x.as_ref() {
                Statement::VariableAssignment(v, e) => {
                    Statement::VariableAssignment(Rc::clone(&variable_map[v]), e.clone())
                },
                _ => x.as_ref().clone(),
            })
        }).collect_vec();

        // The implementation self-injected assmumption functions based on requirements.
        // Now it's time we replace them depending on the actual requirements fulfillment.
        let mut function_replacement_map = HashMap::new();
        for assumption in implementation.requirements_assumption.conformance.values() {
            // TODO Use tail
            let (tail, fulfillment) = &function_binding.requirements_fulfillment.conformance[&assumption.binding.mapping_types(&|type_| type_.unfreezing_any_to_generics())];

            for (abstract_fun, fun_assumption) in assumption.function_mapping.iter() {
                let fun_fulfillment = &fulfillment.function_mapping[abstract_fun];
                function_replacement_map.insert(Rc::clone(fun_assumption), (tail.clone(), Rc::clone(fun_fulfillment)));
            }
        }

        // Find function calls in the expression forest
        for (expression_id, operation) in implementation.expression_forest.operations.iter_mut() {
            match operation {
                ExpressionOperation::FunctionCall(call) => {
                    let resolved_call = resolve_call(call, &generic_replacement_map, &function_replacement_map, &implementation.type_forest);

                    if self.encountered_calls.insert(Rc::clone(&resolved_call)) {
                        self.new_encountered_calls.push(Rc::clone(&resolved_call));
                    }

                    let mono_call: Rc<FunctionBinding> = if !resolved_call.requirements_fulfillment.is_empty() && should_monomorphize(&resolved_call) {
                        self.monomorphize_call(&resolved_call)
                    }
                    else {
                        resolved_call
                    };

                    *operation = ExpressionOperation::FunctionCall(mono_call)
                }
                ExpressionOperation::PairwiseOperations { calls } => {
                    *operation = ExpressionOperation::PairwiseOperations {
                        calls: calls.iter()
                            .map(|call| {
                                let resolved_call = resolve_call(call, &generic_replacement_map, &function_replacement_map, &implementation.type_forest);

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
                    *operation = ExpressionOperation::VariableLookup(Rc::clone(variable_map.get(v).unwrap_or(v)))
                }
                ExpressionOperation::ArrayLiteral => {},
                ExpressionOperation::StringLiteral(_) => {},
            };
        }

        // Update parameter variables
        for param_variable in implementation.parameter_variables.iter_mut() {
            *param_variable = Rc::clone(&variable_map[param_variable])
        }
        implementation.variable_names = implementation.variable_names.drain().map(|(key, value)| {
            (Rc::clone(&variable_map[&key]), value)
        }).collect();

        // Requirements
        // TODO This is correct only if all requirements have been fulfilled.
        //  If monomorphize was requested on a partially generic function, we continue to
        //  have some requirements.
        implementation.requirements_assumption = Box::new(RequirementsAssumption { conformance: Default::default() });

        // Finalize: New identity and head
        implementation.implementation_id = Uuid::new_v4();

        let monomorphized_binding = &self.resolved_call_to_mono_call.get(function_binding).unwrap_or(function_binding);
        implementation.head = Rc::clone(&monomorphized_binding.function);
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

pub fn resolve_call(call: &Rc<FunctionBinding>, generic_replacement_map: &HashMap<Uuid, Box<TypeProto>>, function_replacement_map: &HashMap<Rc<FunctionHead>, (Box<RequirementsFulfillment>, Rc<FunctionHead>)>, type_forest: &TypeForest) -> Rc<FunctionBinding> {
    let default_pair = (RequirementsFulfillment::empty(), Rc::clone(&call.function));
    let (mapped_function_tail, mapped_function) = function_replacement_map.get(&call.function)
        .unwrap_or(&default_pair);
    println!("Call to {:?} with tail {:?}", mapped_function, mapped_function_tail);

    let full_conformance = RequirementsFulfillment::merge(&call.requirements_fulfillment, mapped_function_tail);

    let mut generic_replacement_map: HashMap<Uuid, Box<TypeProto>> = full_conformance.generic_mapping.iter().map(|(any_id, type_)| {
        (*any_id, type_forest.resolve_type(type_).unwrap().replacing_anys(generic_replacement_map))
    }).collect();

    Rc::new(FunctionBinding {
        function: Rc::clone(mapped_function),
        requirements_fulfillment: Box::new(RequirementsFulfillment {
            conformance: full_conformance.conformance.iter()
                .map(|(requirement, (tail, conformance))| {
                    // TODO Use tail
                    (Rc::clone(requirement), (tail.clone(), TraitConformance::new(
                        Rc::clone(requirement),
                        conformance.function_mapping.iter()
                            .map(
                                |(abstract_fun, fulfillment_fun)|
                                (Rc::clone(abstract_fun), Rc::clone(function_replacement_map.get(fulfillment_fun).map(|x| &x.1).unwrap_or_else(|| fulfillment_fun)))
                            )
                            .collect()
                    )))
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
