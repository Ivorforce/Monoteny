use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::rc::Rc;
use itertools::Itertools;
use uuid::Uuid;
use crate::program::allocation::ObjectReference;
use crate::program::calls::FunctionBinding;
use crate::program::computation_tree::{ExpressionForest, ExpressionOperation, Statement};
use crate::program::functions::{Function, FunctionCallType, FunctionInterface, FunctionPointer, Parameter};
use crate::program::generics::TypeForest;
use crate::program::global::FunctionImplementation;
use crate::program::traits::{RequirementsAssumption, RequirementsFulfillment};
use crate::program::types::{TypeProto, TypeUnit};


pub struct FunctionUnfolder {
    pub mapped_calls: HashMap<Rc<FunctionBinding>, Rc<FunctionBinding>>,
    pub new_mappable_calls: Vec<Rc<FunctionBinding>>,
}

impl FunctionUnfolder {
    pub fn new() -> FunctionUnfolder {
        FunctionUnfolder {
            mapped_calls: Default::default(),
            new_mappable_calls: Default::default(),
        }
    }

    pub fn unfold_anonymous(&mut self, implementation: &FunctionImplementation, function_binding: &Rc<FunctionBinding>, should_unfold: &dyn Fn(&Rc<FunctionBinding>) -> bool) -> Rc<FunctionImplementation> {
        // Map types.
        let mut type_forest = implementation.type_forest.clone();

        let generic_replacement_map = &function_binding.requirements_fulfillment.generic_mapping;

        // Change Anys to Generics in the type forest.
        type_forest.bind_any_as_generic(generic_replacement_map).unwrap();

        let mut expression_forest = Box::new(ExpressionForest::new());

        // Map variables.
        // TODO Some could just map Any -> Generic, but some are parameter variables. Those must expose the full type properly.
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
        for (binding, function_resolution) in implementation.requirements_assumption.conformance.iter() {
            for (abstract_fun, fun_placement) in function_resolution.iter() {
                let binds = &function_binding.requirements_fulfillment.conformance[&binding.mapping_types(&|type_| type_.unfreezing_any_to_generics())];
                let replacement = &binds[abstract_fun];
                function_replacement_map.insert(Rc::clone(fun_placement), Rc::clone(replacement));
            }
        }

        // Find function calls in the expression forest
        for (expression_id, operation) in implementation.expression_forest.operations.iter() {
            expression_forest.operations.insert(expression_id.clone(), match operation {
                ExpressionOperation::FunctionCall(call) => {
                    let replaced_pointer = function_replacement_map.get(&call.pointer).unwrap_or(&call.pointer);
                    let replaced_call = Rc::new(FunctionBinding { pointer: Rc::clone(replaced_pointer), requirements_fulfillment: call.requirements_fulfillment.clone() });

                    let unfolded_call: Rc<FunctionBinding> = if should_unfold(&replaced_call) {
                        match self.mapped_calls.entry(Rc::clone(call)) {
                            Entry::Occupied(o) => Rc::clone(o.get()),
                            Entry::Vacant(v) => {
                                self.new_mappable_calls.push(Rc::clone(&replaced_call));
                                Rc::clone(v.insert(map_call(&replaced_call, &generic_replacement_map, &function_replacement_map, &type_forest)))
                            },
                        }
                    }
                    else {
                        replaced_call
                    };

                    ExpressionOperation::FunctionCall(unfolded_call)
                }
                ExpressionOperation::PairwiseOperations { calls } => {
                    ExpressionOperation::PairwiseOperations {
                        calls: calls.iter()
                            .map(|call| {
                                let replaced_pointer = function_replacement_map.get(&call.pointer).unwrap_or(&call.pointer);
                                let replaced_call = Rc::new(FunctionBinding { pointer: Rc::clone(replaced_pointer), requirements_fulfillment: call.requirements_fulfillment.clone() });

                                let unfolded_call: Rc<FunctionBinding> = if should_unfold(&replaced_call) {
                                    match self.mapped_calls.entry(Rc::clone(call)) {
                                        Entry::Occupied(o) => Rc::clone(o.get()),
                                        Entry::Vacant(v) => {
                                            self.new_mappable_calls.push(Rc::clone(&replaced_call));
                                            Rc::clone(v.insert(map_call(&replaced_call, &generic_replacement_map, &function_replacement_map, &type_forest)))
                                        },
                                    }
                                }
                                else {
                                    replaced_call
                                };

                                unfolded_call
                            }).collect_vec()
                    }
                }
                ExpressionOperation::VariableLookup(v) => {
                    ExpressionOperation::VariableLookup(Rc::clone(&variable_map[v]))
                }
                ExpressionOperation::ArrayLiteral => ExpressionOperation::ArrayLiteral,
                ExpressionOperation::StringLiteral(s) => ExpressionOperation::StringLiteral(s.clone()),
            });
        }
        expression_forest.arguments = implementation.expression_forest.arguments.clone();

        Rc::new(FunctionImplementation {
            implementation_id: Uuid::new_v4(),
            pointer: Rc::clone(&function_binding.pointer),  // Re-use premapped pointer
            decorators: implementation.decorators.clone(),
            // TODO Is this correct? No assumptions?
            requirements_assumption: Box::new(RequirementsAssumption { conformance: Default::default() }),
            statements,
            expression_forest,
            type_forest,
            parameter_variables: implementation.parameter_variables.iter().map(|x| Rc::clone(&variable_map[x])).collect_vec(),
            variable_names: implementation.variable_names.clone(),  // Variables don't change with unfolding
        })
    }
}

pub fn map_call(call: &Rc<FunctionBinding>, replacement_map: &HashMap<Uuid, Box<TypeProto>>, function_replacement_map: &HashMap<Rc<FunctionPointer>, Rc<FunctionPointer>>, type_forest: &TypeForest) -> Rc<FunctionBinding> {
    let generic_replacement_map = call.requirements_fulfillment.generic_mapping.iter().map(|(any_id, type_)| {
        (*any_id, type_forest.resolve_type(type_).unwrap().replacing_anys(replacement_map))
    }).collect();

    Rc::new(FunctionBinding {
        pointer: Rc::new(FunctionPointer {
            pointer_id: Uuid::new_v4(),
            target: Rc::new(Function {
                function_id: Uuid::new_v4(),
                interface: Rc::new(map_interface_types(&call.pointer.target.interface, &|x| x.replacing_generics(&generic_replacement_map))),
            }),
            // We're now a static call! (as long as the binding was complete)
            call_type: FunctionCallType::Static,
            name: call.pointer.name.clone(),
            form: call.pointer.form.clone(),
        }),
        requirements_fulfillment: Box::new(RequirementsFulfillment {
            conformance: call.requirements_fulfillment.conformance.iter()
                .map(|(key, mapping)| {
                    (Rc::clone(key), mapping.iter()
                        .map(
                            |(abstract_fun, fulfillment_fun)|
                            (Rc::clone(abstract_fun), Rc::clone(function_replacement_map.get(fulfillment_fun).unwrap_or_else(|| fulfillment_fun)))
                        )
                        .collect()
                    )
                })
                .collect(),
            generic_mapping: generic_replacement_map,
        }),
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
