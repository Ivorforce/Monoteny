use std::rc::Rc;

use crate::error::RResult;
use crate::interpreter::runtime::Runtime;
use crate::program::functions::{FunctionCallExplicity, FunctionHead, FunctionInterface, FunctionLogic, FunctionLogicDescriptor, FunctionRepresentation};
use crate::program::module::Module;
use crate::program::traits::{Trait, TraitConformanceRule};
use crate::program::types::TypeProto;
use crate::resolver::scopes;

pub fn add_trait(runtime: &mut Runtime, module: &mut Module, scope: Option<&mut scopes::Scope>, trait_: &Rc<Trait>) -> RResult<()> {
    let meta_type = TypeProto::one_arg(&runtime.Metatype, TypeProto::unit_struct(trait_));
    let getter = FunctionHead::new_static(
        FunctionInterface::new_provider(&meta_type, vec![]),
        FunctionRepresentation::new_global_implicit(&trait_.name)
    );

    runtime.source.fn_heads.insert(getter.function_id, Rc::clone(&getter));
    runtime.source.trait_references.insert(
        Rc::clone(&getter),
        Rc::clone(trait_),
    );
    runtime.source.fn_logic.insert(
        Rc::clone(&getter),
        FunctionLogic::Descriptor(FunctionLogicDescriptor::TraitProvider(Rc::clone(trait_))),
    );

    if let Some(scope) = scope {
        scope.overload_function(&getter, getter.declared_representation.clone())?;
    }

    module.exposed_functions.insert(getter);

    Ok(())
}

pub fn add_function(runtime: &mut Runtime, module: &mut Module, scope: Option<&mut scopes::Scope>, function: &Rc<FunctionHead>) -> RResult<()> {
    // TODO Once functions are actually objects, we can call add_trait from here.
    let function_trait = Rc::new(Trait::new_with_self(&function.declared_representation.name));
    let conformance_to_function = TraitConformanceRule::manual(runtime.traits.as_ref().unwrap().Function.create_generic_binding(vec![
        ("Self", TypeProto::unit_struct(&function_trait))
    ]), vec![]);
    module.trait_conformance.add_conformance_rule(Rc::clone(&conformance_to_function));

    runtime.source.function_traits.insert(Rc::clone(&function_trait), Rc::clone(&function));

    // The function should be implicit.
    // assert_eq!(function.declared_representation.call_explicity, FunctionCallExplicity::Explicit);
    let getter = FunctionHead::new_static(
        FunctionInterface::new_provider(&TypeProto::unit_struct(&function_trait), vec![]),
        FunctionRepresentation::new(function.declared_representation.name.as_str(), function.declared_representation.target_type, FunctionCallExplicity::Implicit)
    );
    runtime.source.fn_heads.insert(function.function_id, Rc::clone(&function));
    runtime.source.fn_heads.insert(getter.function_id, Rc::clone(&getter));
    runtime.source.fn_logic.insert(
        Rc::clone(&getter),
        FunctionLogic::Descriptor(FunctionLogicDescriptor::FunctionProvider(Rc::clone(&function))),
    );
    runtime.source.fn_getters.insert(Rc::clone(&function), Rc::clone(&getter));

    // Implicits expose themselves, but functions will sit behind a getter
    let exposed_function = function;

    if let Some(scope) = scope {
        scope.overload_function(&exposed_function, exposed_function.declared_representation.clone())?;
        scope.trait_conformance.add_conformance_rule(conformance_to_function);
    }

    module.exposed_functions.insert(exposed_function.clone());

    Ok(())
}
