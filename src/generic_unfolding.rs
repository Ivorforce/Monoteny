use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use itertools::Itertools;
use uuid::Uuid;
use crate::program::allocation::ObjectReference;
use crate::program::calls::FunctionCall;
use crate::program::computation_tree::{ExpressionForest, ExpressionID, ExpressionOperation, Statement};
use crate::program::functions::{Function, FunctionInterface, FunctionPointer, Parameter};
use crate::program::generics::TypeForest;
use crate::program::global::FunctionImplementation;
use crate::program::traits::TraitResolution;
use crate::program::types::TypeProto;


pub struct FunctionUnfolder {
    pub encountered: Vec<Box<FunctionCall>>,
    pub resolved: HashMap<Box<FunctionCall>, Rc<FunctionImplementation>>,
}

impl FunctionUnfolder {
    pub fn new() -> FunctionUnfolder {
        FunctionUnfolder {
            encountered: Default::default(),
            resolved: Default::default(),
        }
    }

    pub fn resolve(&mut self, call: Box<FunctionCall>, implementation: Rc<FunctionImplementation>) -> Rc<FunctionImplementation> {
        let resolved = self.unfold_anonymous(&implementation, &call.resolution);
        self.resolved.insert(call, Rc::clone(&resolved));
        resolved
    }

    pub fn unfold_anonymous(&mut self, fun: &FunctionImplementation, resolution: &TraitResolution) -> Rc<FunctionImplementation> {
        let mut type_replacement_map: HashMap<Uuid, Box<TypeProto>> = Default::default();
        for (requirement, binding) in resolution.requirement_bindings.iter() {
            type_replacement_map.extend(binding.generic_to_type.clone());
        }

        let mut expression_forest = Box::new(ExpressionForest::new());
        let mut type_forest = Box::new(TypeForest::new());

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

        let function_replacement_map = resolution.gather_function_bindings();

        // Find function calls in the expression forest
        for (expression_id, operation) in fun.expression_forest.operations.iter() {
            expression_forest.operations.insert(expression_id.clone(), match operation {
                ExpressionOperation::FunctionCall(call) => {
                    let new_pointer = function_replacement_map.get(&call.pointer).unwrap_or(&call.pointer);
                    if !call.resolution.requirement_bindings.is_empty() {
                        todo!("Unfold function")
                    } else {
                        let call = FunctionCall {
                            pointer: Rc::clone(new_pointer),
                            resolution: TraitResolution::new()
                        };

                        self.encountered.push(Box::new(call.clone()));
                        ExpressionOperation::FunctionCall(call)
                    }
                }
                ExpressionOperation::PairwiseOperations { calls } => {
                    ExpressionOperation::PairwiseOperations {
                        calls: calls.iter()
                            .map(|call| {
                                let new_pointer = function_replacement_map.get(&call.pointer).unwrap_or(&call.pointer);

                                if !call.resolution.requirement_bindings.is_empty() {
                                    todo!("Unfold function")
                                } else {
                                    let call = FunctionCall {
                                        pointer: Rc::clone(new_pointer),
                                        resolution: TraitResolution::new()
                                    };

                                    self.encountered.push(Box::new(call.clone()));
                                    call
                                }
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

        // Insert the finished function to the vector
        Rc::new(FunctionImplementation {
            implementation_id: Uuid::new_v4(),
            pointer: Rc::new(FunctionPointer {
                pointer_id: fun.pointer.pointer_id.clone(),
                target: Rc::new(Function {
                    function_id: Uuid::new_v4(),
                    interface: Rc::new(map_interface(&fun.pointer.target.interface, &type_replacement_map, &variable_map)),
                }),
                call_type: fun.pointer.call_type.clone(),
                name: fun.pointer.name.clone(),
                form: fun.pointer.form.clone(),
            }),
            decorators: fun.decorators.clone(),
            conformance_delegations: HashMap::new(),
            statements,
            expression_forest,
            type_forest,
            variable_names: fun.variable_names.clone(),  // Variables don't change with unfolding
        })
    }
}

pub fn map_variable(variable: &ObjectReference, type_replacement_map: &HashMap<Uuid, Box<TypeProto>>) -> Rc<ObjectReference> {
    Rc::new(ObjectReference {
        id: variable.id.clone(),
        type_: variable.type_.replacing_any(type_replacement_map),
        mutability: variable.mutability.clone(),
    })
}

pub fn map_interface(interface: &FunctionInterface, type_replacement_map: &HashMap<Uuid, Box<TypeProto>>, object_replacement_map: &HashMap<Rc<ObjectReference>, Rc<ObjectReference>>) -> FunctionInterface {
    FunctionInterface {
        parameters: interface.parameters.iter().map(|x| Parameter {
            external_key: x.external_key.clone(),
            internal_name: x.internal_name.clone(),
            target: Rc::clone(object_replacement_map.get(&x.target).unwrap_or(&x.target)),
        }).collect(),
        return_type: interface.return_type.replacing_any(type_replacement_map),
        requirements: vec![],  // TODO
    }
}
