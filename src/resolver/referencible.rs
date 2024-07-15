use std::rc::Rc;

use crate::error::RResult;
use crate::interpreter::runtime::Runtime;
use crate::program::function_object::{FunctionCallExplicity, FunctionRepresentation, FunctionTargetType};
use crate::program::function_pointer::FunctionPointer;
use crate::program::functions::{FunctionHead, FunctionInterface};
use crate::program::global::{FunctionLogic, FunctionLogicDescriptor};
use crate::program::module::Module;
use crate::program::traits::{Trait, TraitConformanceRule};
use crate::program::types::TypeProto;
use crate::resolver::scopes;

pub fn add_trait(runtime: &mut Runtime, module: &mut Module, scope: Option<&mut scopes::Scope>, trait_: &Rc<Trait>) -> RResult<()> {
    let meta_type = TypeProto::one_arg(&runtime.Metatype, TypeProto::unit_struct(trait_));
    let getter = FunctionHead::new_static(FunctionInterface::new_provider(&meta_type, vec![]));

    runtime.source.fn_heads.insert(getter.function_id, Rc::clone(&getter));
    runtime.source.trait_references.insert(
        Rc::clone(&getter),
        Rc::clone(trait_),
    );
    runtime.source.fn_logic.insert(
        Rc::clone(&getter),
        FunctionLogic::Descriptor(FunctionLogicDescriptor::TraitProvider(Rc::clone(trait_))),
    );

    let representation = FunctionRepresentation::new(&trait_.name, FunctionTargetType::Global, FunctionCallExplicity::Implicit);

    runtime.source.fn_representations.insert(
        Rc::clone(&getter),
        representation.clone(),
    );

    if let Some(scope) = scope {
        scope.overload_function(&getter, representation)?;
    }

    module.exposed_functions.insert(getter);

    Ok(())
}

pub fn add_function(runtime: &mut Runtime, module: &mut Module, scope: Option<&mut scopes::Scope>, function: &FunctionPointer) -> RResult<()> {
    // TODO Once functions are actually objects, we can call add_trait from here.
    let function_trait = Rc::new(Trait::new_with_self(&function.representation.name));
    let conformance_to_function = TraitConformanceRule::manual(runtime.traits.as_ref().unwrap().Function.create_generic_binding(vec![
        ("Self", TypeProto::unit_struct(&function_trait))
    ]), vec![]);
    module.trait_conformance.add_conformance_rule(Rc::clone(&conformance_to_function));

    runtime.source.function_traits.insert(Rc::clone(&function_trait), Rc::clone(&function.target));

    let getter = FunctionHead::new_static(
        FunctionInterface::new_provider(&TypeProto::unit_struct(&function_trait), vec![]),
    );
    runtime.source.fn_heads.insert(function.target.function_id, Rc::clone(&function.target));
    runtime.source.fn_heads.insert(getter.function_id, Rc::clone(&getter));
    runtime.source.fn_logic.insert(
        Rc::clone(&getter),
        FunctionLogic::Descriptor(FunctionLogicDescriptor::FunctionProvider(Rc::clone(&function.target))),
    );
    runtime.source.fn_getters.insert(Rc::clone(&function.target), Rc::clone(&getter));

    runtime.source.fn_representations.insert(Rc::clone(&function.target), function.representation.clone());

    runtime.source.fn_representations.insert(
        Rc::clone(&getter),
        FunctionRepresentation::new(function.representation.name.as_str(), function.representation.target_type, FunctionCallExplicity::Implicit)
    );

    // Implicits expose themselves, but functions will sit behind a getter
    let exposed_function = function;

    if let Some(scope) = scope {
        scope.overload_function(&exposed_function.target, runtime.source.fn_representations[&exposed_function.target].clone())?;
        scope.trait_conformance.add_conformance_rule(conformance_to_function);
    }

    module.exposed_functions.insert(exposed_function.target.clone());

    Ok(())
}
