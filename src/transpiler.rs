use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::DerefMut;
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use crate::constant_folding::ConstantFold;
use crate::error::{RResult, RuntimeError};
use crate::interpreter;
use crate::interpreter::Runtime;
use crate::monomorphize::Monomorphizer;
use crate::program::calls::FunctionBinding;
use crate::program::functions::FunctionHead;
use crate::program::global::FunctionImplementation;
use crate::program::module::Module;
use crate::program::traits::RequirementsFulfillment;

pub mod python;
pub mod namespaces;

pub struct Transpiler {
    // In the future, this should all be accessible by monoteny code itself - including the context.
    pub monomorphizer: Box<Monomorphizer>,
    pub main_function: Option<Rc<FunctionHead>>,
    pub exported_functions: Vec<Box<FunctionImplementation>>,
    pub internal_functions: Vec<Box<FunctionImplementation>>,
}

pub trait Context {
    fn builtin_functions(&self) -> HashSet<Rc<FunctionHead>>;

    fn make_files(&self, filename: &str, runtime: &Runtime, transpiler: &Transpiler) -> RResult<HashMap<String, String>>;
}

pub fn run(module: &Module, runtime: &mut Runtime, context: &mut impl Context) -> RResult<Transpiler> {
    let builtin_functions = context.builtin_functions();

    let transpiler = Transpiler {
        monomorphizer: Box::new(Monomorphizer::new()),
        main_function: module.main_functions.iter().at_most_one()
            .map_err(|_| RuntimeError::new(format!("Too many @main functions declared: {:?}", module.main_functions)))?
            .cloned(),
        exported_functions: vec![],
        internal_functions: vec![],
    };

    let transpiler = Rc::new(RefCell::new(transpiler));

    interpreter::run::transpile(module, runtime, &|implementation_id, runtime| {
        let mut transpiler = transpiler.borrow_mut();
        let transpiler_context = transpiler.deref_mut();
        let implementation = &runtime.source.fn_implementations[&implementation_id];

        if !implementation.head.interface.generics.is_empty() {
            // We'll need to somehow transpile requirements as protocols and generics as generics.
            // That's for later!
            panic!("Transpiling generic functions is not supported yet: {:?}", implementation.head);
        }

        let mut mono_implementation = implementation.clone();
        transpiler_context.monomorphizer.monomorphize_function(
            &mut mono_implementation,
            &Rc::new(FunctionBinding {
                // The implementation's pointer is fine.
                function: Rc::clone(&implementation.head),
                // The resolution SHOULD be empty: The function is transpiled WITH its generics.
                // Unless generics are bound in the transpile directive, which is TODO
                requirements_fulfillment: RequirementsFulfillment::empty(),
            }),
            &|f| !builtin_functions.contains(&f.function)
        );

        transpiler_context.exported_functions.push(mono_implementation);
    })?;

    // Find and monomorphize internal symbols
    let mut transpiler = Rc::try_unwrap(transpiler).map_err(|_| ()).expect("Internal Error on try_unwrap(transpiler)").into_inner();

    while let Some(function_binding) = transpiler.monomorphizer.new_encountered_calls.pop() {
        guard!(let Some(implementation) = runtime.source.fn_implementations.get(&function_binding.function) else {
            // We don't have an implementation ready, so it must be a core or otherwise injected.
            continue;
        });

        // We may not create a new one through monomorphization, but we still need to take ownership.
        let mut mono_implementation = implementation.clone();
        // If the call had an empty fulfillment, it wasn't monomorphized. We can just use the implementation itself!
        if transpiler.monomorphizer.resolved_call_to_mono_call.contains_key(&function_binding) {
            transpiler.monomorphizer.monomorphize_function(
                &mut mono_implementation,
                &function_binding,
                &|f| !builtin_functions.contains(&f.function)
            );
        };

        transpiler.internal_functions.push(mono_implementation);
    }

    // TODO We should sort the internal functions. This could be done roughly by putting them in the
    //  order the player defined it - which leaves only different monomorpizations to be sorted.
    //  Those can be sorted by something like the displayed 'function to string' (without randomized uuid).
    //  This should work because two traits sharing the same name but being different IDs should be rare.
    //  In that rare case, we can probably live with being indeterministic.

    // I should note that 0 parameter functions should NOT create circles - although they may if they're
    //  badly implemented. This should be caught by the same sorter and throw if it happens.

    Ok(transpiler)
}

pub fn constant_fold(transpiler: &mut Transpiler) {
    // Run constant folder
    let mut constant_folder = ConstantFold::new();
    let exported_function_order = transpiler.exported_functions.iter().map(|x| Rc::clone(&x.head)).collect_vec();

    // The exported functions aren't called so it makes sense to prepare the internal functions first.
    for implementation in transpiler.internal_functions.drain(..) {
        constant_folder.add(implementation, true);
    }
    for implementation in transpiler.exported_functions.drain(..) {
        constant_folder.add(implementation, false);
    }

    // Exported functions MUST be there still, because they can't be inlined.
    transpiler.exported_functions.extend(exported_function_order.iter().map(|x| constant_folder.implementation_by_head.remove(x).unwrap()).collect_vec());
    // The order of the internal functions is unimportant anyway, because they are sorted later.
    transpiler.internal_functions = constant_folder.drain_all_functions_yield_uninlined();
}
