use std::collections::HashMap;
use std::rc::Rc;

use itertools::Itertools;
use linked_hash_set::LinkedHashSet;

use crate::program::allocation::ObjectReference;
use crate::program::expression_tree::ExpressionOperation;
use crate::program::functions::{FunctionBinding, FunctionHead, FunctionImplementation, FunctionInterface, FunctionType, Parameter};
use crate::program::generics::TypeForest;
use crate::program::traits::{RequirementsAssumption, RequirementsFulfillment, Trait, TraitConformanceWithTail};
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

    // Find function calls in the expression forest
    for expression_id in implementation.expression_tree.deep_children(implementation.expression_tree.root) {
        let mut operation = implementation.expression_tree.values.get_mut(&expression_id).unwrap();

        match operation {
            ExpressionOperation::FunctionCall(call) => {
                let resolved_call = resolve_call(call, &function_binding.requirements_fulfillment, &generic_replacement_map, &implementation.type_forest);
                encountered_calls.insert_if_absent(Rc::clone(&resolved_call));
                *operation = ExpressionOperation::FunctionCall(resolved_call)
            }
            ExpressionOperation::PairwiseOperations { calls } => {
                *operation = ExpressionOperation::PairwiseOperations {
                    calls: calls.iter()
                        .map(|call| {
                            let resolved_call = resolve_call(call, &function_binding.requirements_fulfillment, &generic_replacement_map, &implementation.type_forest);

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
            ExpressionOperation::IfThenElse => {}
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

pub fn resolve_call(call: &Rc<FunctionBinding>, context: &RequirementsFulfillment, generic_replacement_map: &HashMap<Rc<Trait>, Rc<TypeProto>>, type_forest: &TypeForest) -> Rc<FunctionBinding> {
    // A function can have multiple requirements. They must be fully fulfilled after monomorphization.
    // Each requirement has two routes it can be fulfilled from:
    // 1) The caller has already fulfilled the requirement, and it is passed here in the function replacement map as its tail.
    // 2) The requirement was exposed to our function in an abstract way, and we had to fulfill it.

    // Source 2)
    let mut requirements_fulfillment = map_requirements_fulfillment(
        &call.requirements_fulfillment,
        context,
        generic_replacement_map,
        type_forest
    );

    let function: Rc<FunctionHead>;
    if let FunctionType::Polymorphic { assumed_requirement, abstract_function } = &call.function.function_type {
        let TraitConformanceWithTail {conformance, tail} = context.conformance[assumed_requirement].as_ref();

        function = Rc::clone(&conformance.function_mapping[abstract_function]);

        if !tail.is_empty() {
            // TODO I think this is correct?
            // The caller's requirements fulfillment needs to bring along a tail.
            // We'll attach this tail to our requirements fulfillment as source 1).
            requirements_fulfillment.conformance.extend(tail.conformance.clone());
            requirements_fulfillment.generic_mapping.extend(tail.generic_mapping.clone());
        }
    }
    else {
        function = Rc::clone(&call.function)
    }

    return Rc::new(FunctionBinding {
        function,
        requirements_fulfillment: Rc::new(requirements_fulfillment),
    })
}

fn map_requirements_fulfillment(rc: &Rc<RequirementsFulfillment>, context: &RequirementsFulfillment, generic_replacement_map: &HashMap<Rc<Trait>, Rc<TypeProto>>, type_forest: &TypeForest) -> RequirementsFulfillment {
    // A requirements fulfillment (for a function call) consists of many conformances to requirements.
    // Every conformance either:
    // 1) Uses some global conformance declaration. In this case, it's already correct - except for
    //     its tail, which needs to be checked for replacements too.
    // 2) Uses a conformances that was created by the assumption of this function's requirement.
    //     In this case, we merely need to use whatever is being injected into us. Since it's coming
    //     from outside, it's already correct and does not need to be mapped.
    //
    // Luckily, it is easy to find out which category each conformance belongs to. We simply need to
    //  check if the fulfillment we're being provided contains the conformance we're using.
    //  For example, if we're using Animal<Human>in this call, and the context is providing a
    //  fulfillment for Animal<Human>, we can simply copy the fulfillment's version.
    RequirementsFulfillment {
        conformance: rc.conformance.iter()
            .map(|(requirement, conformance)| {
                return (
                    Rc::clone(requirement),
                    if let Some(replacement) = context.conformance.get(&conformance.conformance.binding) {
                        // Conformance was abstract / has been mapped by the caller.
                        Rc::clone(replacement)
                    } else {
                        if conformance.tail.is_empty() {
                            // Conformance is static / good as-is.
                            Rc::clone(conformance)
                        } else {
                            // Conformance is static.
                            // We still need to map its tail because it may use requirements assumptions.
                            Rc::new(TraitConformanceWithTail {
                                conformance: Rc::clone(&conformance.conformance),
                                tail: Rc::new(
                                    map_requirements_fulfillment(&conformance.tail, context, generic_replacement_map, type_forest)
                                ),
                            })
                        }
                    }
                )
            })
            .collect(),
        generic_mapping: rc.generic_mapping.iter().map(|(trait_, type_)| {
            (Rc::clone(trait_), type_forest.resolve_type(type_).unwrap().replacing_structs(generic_replacement_map))
        }).collect(),
    }
}

pub fn monomorphize_head(binding: &FunctionBinding) -> Rc<FunctionHead> {
    FunctionHead::new(
        Rc::new(map_interface_types(&binding.function.interface, &binding.requirements_fulfillment.generic_mapping)),
        binding.function.function_type.clone(),
        binding.function.declared_representation.clone(),
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
