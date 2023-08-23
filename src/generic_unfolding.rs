use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::rc::Rc;
use itertools::Itertools;
use uuid::Uuid;
use crate::program::allocation::ObjectReference;
use crate::program::calls::FunctionBinding;
use crate::program::computation_tree::{ExpressionForest, ExpressionOperation, Statement};
use crate::program::functions::{Function, FunctionInterface, FunctionPointer, Parameter};
use crate::program::global::FunctionImplementation;
use crate::program::traits::TraitResolution;
use crate::program::types::TypeProto;


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

    pub fn unfold_anonymous(&mut self, fun: &FunctionImplementation, function_binding: &Rc<FunctionBinding>, should_unfold: &dyn Fn(&Rc<FunctionBinding>) -> bool) -> Rc<FunctionImplementation> {
        // Map types.
        let mut type_forest = fun.type_forest.clone();

        let mut type_replacement_map: HashMap<Uuid, Box<TypeProto>> = Default::default();
        for (binding, function_mapping) in function_binding.resolution.conformance.iter() {
            type_replacement_map.extend(binding.generic_to_type.clone());

            for (generic, type_) in binding.generic_to_type.iter() {
                type_forest.bind(*generic, type_).unwrap();
            }
        }

        let mut expression_forest = Box::new(ExpressionForest::new());

        // Map variables, to change the types.
        let variable_map: HashMap<Rc<ObjectReference>, Rc<ObjectReference>> = fun.variable_names.keys()
            .map(|v| (Rc::clone(v), map_variable(v, &type_replacement_map)))
            .collect();

        // Map statements. Expressions are mapped elsewhere, so this should be easy.
        let statements = fun.statements.iter().map(|x| {
            Box::new(match x.as_ref() {
                Statement::VariableAssignment(v, e) => {
                    Statement::VariableAssignment(Rc::clone(&variable_map[v]), e.clone())
                },
                _ => x.as_ref().clone(),
            })
        }).collect_vec();

        let mut function_replacement_map = HashMap::new();
        for (binding, function_resolution) in fun.assumed_requirements.conformance.iter() {
            for (abstract_fun, fun_placement) in function_resolution.iter() {
                let replacement = &function_binding.resolution.conformance[binding][abstract_fun];
                function_replacement_map.insert(Rc::clone(fun_placement), Rc::clone(replacement));
            }
        }
        println!("Fun replacement: {:?} / {:?}  --------- {:?}", fun.pointer, function_binding.pointer, function_replacement_map);

        // Find function calls in the expression forest
        for (expression_id, operation) in fun.expression_forest.operations.iter() {
            expression_forest.operations.insert(expression_id.clone(), match operation {
                ExpressionOperation::FunctionCall(call) => {
                    let replaced_pointer = function_replacement_map.get(&call.pointer).unwrap_or(&call.pointer);
                    println!("Pre-Call: {:?}", call.pointer);
                    let replaced_call = Rc::new(FunctionBinding { pointer: Rc::clone(replaced_pointer), resolution: call.resolution.clone() });
                    println!("Replaced Call: {:?}", replaced_pointer);

                    let unfolded_call: Rc<FunctionBinding> = if should_unfold(&replaced_call) {
                        match self.mapped_calls.entry(Rc::clone(call)) {
                            Entry::Occupied(o) => Rc::clone(o.get()),
                            Entry::Vacant(v) => {
                                self.new_mappable_calls.push(Rc::clone(&replaced_call));
                                Rc::clone(v.insert(map_call(&replaced_call)))
                            },
                        }
                    }
                    else {
                        replaced_call
                    };
                    println!("Unfolded: {:?}", unfolded_call.pointer);

                    ExpressionOperation::FunctionCall(unfolded_call)
                }
                ExpressionOperation::PairwiseOperations { calls } => {
                    ExpressionOperation::PairwiseOperations {
                        calls: calls.iter()
                            .map(|call| {
                                let replaced_pointer = function_replacement_map.get(&call.pointer).unwrap_or(&call.pointer);
                                let replaced_call = Rc::new(FunctionBinding { pointer: Rc::clone(replaced_pointer), resolution: call.resolution.clone() });

                                let unfolded_call: Rc<FunctionBinding> = if should_unfold(&replaced_call) {
                                    match self.mapped_calls.entry(Rc::clone(call)) {
                                        Entry::Occupied(o) => Rc::clone(o.get()),
                                        Entry::Vacant(v) => {
                                            self.new_mappable_calls.push(Rc::clone(&replaced_call));
                                            Rc::clone(v.insert(map_call(&replaced_call)))
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
        expression_forest.arguments = fun.expression_forest.arguments.clone();

        Rc::new(FunctionImplementation {
            implementation_id: Uuid::new_v4(),
            pointer: Rc::clone(&function_binding.pointer),  // Re-use premapped pointer
            decorators: fun.decorators.clone(),
            assumed_requirements: TraitResolution::new(),
            statements,
            expression_forest,
            type_forest,
            parameter_variables: fun.parameter_variables.iter().map(|x| Rc::clone(&variable_map[x])).collect_vec(),
            variable_names: fun.variable_names.clone(),  // Variables don't change with unfolding
        })
    }
}

pub fn map_call(call: &Rc<FunctionBinding>) -> Rc<FunctionBinding> {
    let mut type_replacement_map: HashMap<Uuid, Box<TypeProto>> = Default::default();
    for (binding, function_mapping) in call.resolution.conformance.iter() {
        type_replacement_map.extend(binding.generic_to_type.clone());
    }

    Rc::new(FunctionBinding {
        pointer: Rc::new(FunctionPointer {
            pointer_id: Uuid::new_v4(),
            target: Rc::new(Function {
                function_id: Uuid::new_v4(),
                interface: Rc::new(map_interface(&call.pointer.target.interface, &type_replacement_map)),
            }),
            call_type: call.pointer.call_type.clone(),
            name: call.pointer.name.clone(),
            form: call.pointer.form.clone(),
        }),
        resolution: call.resolution.clone(), // TODO This should be empty now? Maybe?
    })
}

pub fn map_variable(variable: &ObjectReference, type_replacement_map: &HashMap<Uuid, Box<TypeProto>>) -> Rc<ObjectReference> {
    Rc::new(ObjectReference {
        id: variable.id.clone(),
        type_: variable.type_.replacing_any(type_replacement_map),
        mutability: variable.mutability.clone(),
    })
}

pub fn map_interface(interface: &FunctionInterface, type_replacement_map: &HashMap<Uuid, Box<TypeProto>>) -> FunctionInterface {
    FunctionInterface {
        parameters: interface.parameters.iter().map(|x| Parameter {
            external_key: x.external_key.clone(),
            internal_name: x.internal_name.clone(),
            type_: x.type_.replacing_any(type_replacement_map),
        }).collect(),
        return_type: interface.return_type.replacing_any(type_replacement_map),
        requirements: interface.requirements.iter().map(|x| x.mapping_types(&|x| x.replacing_any(type_replacement_map))).collect(),
    }
}
