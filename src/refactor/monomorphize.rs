use std::collections::HashMap;
use std::rc::Rc;
use itertools::Itertools;
use linked_hash_set::LinkedHashSet;
use crate::program::allocation::ObjectReference;
use crate::program::calls::FunctionBinding;
use crate::program::expression_tree::{ExpressionOperation};
use crate::program::functions::{FunctionHead, FunctionInterface, Parameter};
use crate::program::generics::TypeForest;
use crate::program::global::FunctionImplementation;
use crate::program::traits::{RequirementsAssumption, RequirementsFulfillment, Trait, TraitConformance, TraitConformanceWithTail};
use crate::program::types::TypeProto;

pub fn monomorphize_implementation(implementation: &mut FunctionImplementation, function_binding: &FunctionBinding) -> LinkedHashSet<Rc<FunctionBinding>> {
    let mut encountered_calls = LinkedHashSet::new();

    // Map types.
    let generic_replacement_map = &function_binding.requirements_fulfillment.generic_mapping;

    // Change Anys to Generics in the type forest.
    implementation.type_forest.rebind_structs_as_generic(generic_replacement_map).unwrap();

    // Map variables.
    // TODO For fully internal variables, it would be enough to set the type to the Any's corresponding Generic,
    //  because those have been bound in the type forest. For variables featured in the interface, however, the
    //  type must be properly resolved. So we might as well map all variables to resolved types.
    let locals_map: HashMap<Rc<ObjectReference>, Rc<ObjectReference>> = implementation.locals_names.keys()
        .map(|v| {
            (Rc::clone(v), map_variable(v, &implementation.type_forest, &generic_replacement_map))
        })
        .collect();

    // The implementation self-injected assmumption functions based on requirements.
    // Now it's time we replace them depending on the actual requirements fulfillment.
    let mut function_replacement_map = HashMap::new();
    for assumption in implementation.requirements_assumption.conformance.values() {
        // TODO Use tail..?
        let mapped_assumption = &assumption.binding;
        let conformance = &function_binding.requirements_fulfillment.conformance[mapped_assumption];

        for (abstract_fun, fun_assumption) in assumption.function_mapping.iter() {
            let fun_fulfillment = &conformance.conformance.function_mapping[abstract_fun];
            function_replacement_map.insert(Rc::clone(fun_assumption), (conformance.tail.clone(), Rc::clone(fun_fulfillment)));
        }
    }

    // Find function calls in the expression forest
    for expression_id in implementation.expression_tree.deep_children(implementation.expression_tree.root) {
        let mut operation = implementation.expression_tree.values.get_mut(&expression_id).unwrap();

        match operation {
            ExpressionOperation::FunctionCall(call) => {
                let resolved_call = resolve_call(call, &generic_replacement_map, &function_replacement_map, &implementation.type_forest);
                encountered_calls.insert_if_absent(Rc::clone(&resolved_call));
                *operation = ExpressionOperation::FunctionCall(resolved_call)
            }
            ExpressionOperation::PairwiseOperations { calls } => {
                *operation = ExpressionOperation::PairwiseOperations {
                    calls: calls.iter()
                        .map(|call| {
                            let resolved_call = resolve_call(call, &generic_replacement_map, &function_replacement_map, &implementation.type_forest);

                            encountered_calls.insert_if_absent(Rc::clone(&resolved_call));

                            resolved_call
                        }).collect_vec()
                }
            }
            ExpressionOperation::GetLocal(v) => {
                // If we cannot find a replacement, it's a static variable. Unless we have a bug.
                *operation = ExpressionOperation::GetLocal(Rc::clone(locals_map.get(v).unwrap_or(v)))
            }
            ExpressionOperation::SetLocal(v) => {
                *operation = ExpressionOperation::SetLocal(Rc::clone(locals_map.get(v).unwrap_or(v)))
            }
            ExpressionOperation::ArrayLiteral => {},
            ExpressionOperation::StringLiteral(_) => {},
            ExpressionOperation::Block => {},
            ExpressionOperation::Return => {}
        };
    }

    // Update parameter variables
    for param_variable in implementation.parameter_locals.iter_mut() {
        *param_variable = Rc::clone(&locals_map[param_variable])
    }
    implementation.locals_names = implementation.locals_names.drain().map(|(key, value)| {
        (Rc::clone(&locals_map[&key]), value)
    }).collect();

    // Requirements
    // TODO This is correct only if all requirements have been fulfilled.
    //  If monomorphize was requested on a partially generic function, we continue to
    //  have some requirements.
    implementation.requirements_assumption = Box::new(RequirementsAssumption { conformance: Default::default() });
    implementation.head = monomorphize_head(function_binding);

    encountered_calls
}

pub fn resolve_call(call: &Rc<FunctionBinding>, generic_replacement_map: &HashMap<Rc<Trait>, Rc<TypeProto>>, function_replacement_map: &HashMap<Rc<FunctionHead>, (Rc<RequirementsFulfillment>, Rc<FunctionHead>)>, type_forest: &TypeForest) -> Rc<FunctionBinding> {
    let default_pair = (RequirementsFulfillment::empty(), Rc::clone(&call.function));
    let (mapped_function_tail, mapped_function) = function_replacement_map.get(&call.function)
        .unwrap_or(&default_pair);

    let full_conformance = RequirementsFulfillment::merge(&call.requirements_fulfillment, mapped_function_tail);

    let generic_replacement_map: HashMap<Rc<Trait>, Rc<TypeProto>> = full_conformance.generic_mapping.iter().map(|(trait_, type_)| {
        (Rc::clone(trait_), type_forest.resolve_type(type_).unwrap().replacing_structs(generic_replacement_map))
    }).collect();

    Rc::new(FunctionBinding {
        function: Rc::clone(mapped_function),
        requirements_fulfillment: Rc::new(RequirementsFulfillment {
            conformance: full_conformance.conformance.iter()
                .map(|(requirement, conformance)| {
                    // TODO Use / map tail?
                    (Rc::clone(requirement), Rc::new(TraitConformanceWithTail {
                        tail: conformance.tail.clone(),
                        conformance: TraitConformance::new(
                            // Don't map the requirements
                            //  - there MIGHT be two different requirements like "A is Float" and "B is Float",
                            //  which map to map to A 'Float32 and B 'Float32, but are fulfilled differently.
                            //  Granted, this is a rare use-case, but it's #valid nonetheless.
                            //  So, the requirements must be mapped as-is.
                            Rc::clone(requirement),
                            conformance.conformance.function_mapping.iter()
                                .map(
                                    |(abstract_fun, fulfillment_fun)|
                                    (Rc::clone(abstract_fun), Rc::clone(function_replacement_map.get(fulfillment_fun).map(|x| &x.1).unwrap_or_else(|| fulfillment_fun)))
                                )
                                .collect()
                        )
                    }))
                })
                .collect(),
            generic_mapping: generic_replacement_map,
        }),
    })
}

pub fn monomorphize_head(binding: &FunctionBinding) -> Rc<FunctionHead> {
    FunctionHead::new(
        Rc::new(map_interface_types(&binding.function.interface, &binding.requirements_fulfillment.generic_mapping)),
        binding.function.function_type.clone(),
    )
}

pub fn map_variable(variable: &ObjectReference, type_forest: &TypeForest, type_replacement_map: &HashMap<Rc<Trait>, Rc<TypeProto>>) -> Rc<ObjectReference> {
    Rc::new(ObjectReference {
        id: variable.id.clone(),
        type_: type_forest.resolve_type(&variable.type_).unwrap().replacing_structs(type_replacement_map),
        mutability: variable.mutability.clone(),
    })
}

pub fn map_interface_types(interface: &FunctionInterface, mapping: &HashMap<Rc<Trait>, Rc<TypeProto>>) -> FunctionInterface {
    FunctionInterface {
        parameters: interface.parameters.iter().map(|x| Parameter {
            external_key: x.external_key.clone(),
            internal_name: x.internal_name.clone(),
            type_: x.type_.replacing_structs(mapping),
        }).collect(),
        return_type: interface.return_type.replacing_structs(mapping),
        requirements: interface.requirements.iter().map(|x| x.mapping_types(&|type_| type_.replacing_structs(mapping))).collect(),
        // TODO Not sure if this is correct - if the mapping introduces MORE generics again, the new
        //  value is wrong. Luckily, this is not a use-case of ours for now - it will only be relevant
        //  when generic transpilation is allowed.
        generics: interface.generics.iter().filter(|(k, v)| !mapping.contains_key(*v)).map(|(a, b)| (a.clone(), b.clone())).collect(),
    }
}
