pub mod types;
pub mod builtins;
pub mod class;
pub mod ast;
pub mod optimization;
pub mod imperative;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::io::Write;
use std::ops::DerefMut;
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use uuid::Uuid;
use regex;
use crate::constant_folding::ConstantFold;
use crate::monomorphize::Monomorphizer;
use crate::interpreter;
use crate::interpreter::{Runtime, InterpreterError};

use crate::program::computation_tree::*;
use crate::program::calls::FunctionBinding;
use crate::program::functions::FunctionHead;
use crate::program::global::{BuiltinFunctionHint, FunctionImplementation};
use crate::program::module::Module;
use crate::program::traits::RequirementsFulfillment;
use crate::program::types::TypeUnit;
use crate::transpiler::namespaces;
use crate::transpiler::python::class::{ClassContext, transpile_class};
use crate::transpiler::python::imperative::{FunctionContext, transpile_function};
use crate::transpiler::python::optimization::TranspilationHint;


pub struct TranspilerContext {
    monomorphizer: Box<Monomorphizer>,
    exported_functions: Vec<Box<FunctionImplementation>>,
    internal_functions: Vec<Box<FunctionImplementation>>,
    fn_transpilation_hints: HashMap<Rc<FunctionHead>, TranspilationHint>,
}

pub fn transpile_module(module: &Module, runtime: &mut Runtime, should_constant_fold: bool) -> Result<Box<ast::Module>, InterpreterError> {
    let transpiler_context = TranspilerContext {
        monomorphizer: Box::new(Monomorphizer::new()),
        exported_functions: vec![],
        internal_functions: vec![],
        fn_transpilation_hints: optimization::prepare(runtime),
    };

    let strictly_polymorphic_functions: HashSet<_> = runtime.source.fn_builtin_hints.keys()
        .chain(transpiler_context.fn_transpilation_hints.keys()).cloned().collect();

    let transpiler_context = Rc::new(RefCell::new(transpiler_context));

    // Run interpreter

    interpreter::run::transpile(module, runtime, &|implementation_id, runtime| {
        let mut transpiler_context = transpiler_context.borrow_mut();
        let transpiler_context = transpiler_context.deref_mut();
        let implementation = &runtime.source.fn_implementations[&implementation_id];

        if implementation.head.interface.collect_generics().len() > 0 {
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
            &|f| !strictly_polymorphic_functions.contains(&f.function)
        );

        transpiler_context.exported_functions.push(mono_implementation);
    })?;

    // Find and monomorphize internal symbols
    let mut transpiler_context = transpiler_context.borrow_mut();
    let mut transpiler_context = transpiler_context.deref_mut();

    while let Some(function_binding) = transpiler_context.monomorphizer.new_encountered_calls.pop() {
        guard!(let Some(implementation) = runtime.source.fn_implementations.get(&function_binding.function) else {
            // We don't have an implementation ready, so it must be a builtin or otherwise injected.
            continue;
        });

        // We may not create a new one through monomorphization, but we still need to take ownership.
        let mut mono_implementation = implementation.clone();
        // If the call had an empty fulfillment, it wasn't monomorphized. We can just use the implementation itself!
        if transpiler_context.monomorphizer.resolved_call_to_mono_call.contains_key(&function_binding) {
            transpiler_context.monomorphizer.monomorphize_function(
                &mut mono_implementation,
                &function_binding,
                &|f| !strictly_polymorphic_functions.contains(&f.function)
            );
        };

        transpiler_context.internal_functions.push(mono_implementation);
    }

    if should_constant_fold && false {
        // Run constant folder
        let mut constant_folder = ConstantFold::new();
        let internal_function_order = transpiler_context.internal_functions.iter().map(|x| Rc::clone(&x.head)).collect_vec();
        let exported_function_order = transpiler_context.exported_functions.iter().map(|x| Rc::clone(&x.head)).collect_vec();

        for implementation in transpiler_context.internal_functions.drain(..) {
            constant_folder.add(implementation, true);
        }
        for implementation in transpiler_context.exported_functions.drain(..) {
            constant_folder.add(implementation, false);
        }
    }

    finalize(module, &transpiler_context, runtime)
}

pub fn finalize(module: &Module, transpiler_context: &TranspilerContext, runtime: &Runtime) -> Result<Box<ast::Module>, InterpreterError> {
    let mut struct_ids = HashMap::new();

    let mut global_namespace = builtins::create_name_level(&runtime.builtins, &mut struct_ids);
    let builtin_structs: HashSet<_> = struct_ids.keys().map(Clone::clone).collect();
    let mut file_namespace = global_namespace.add_sublevel();
    let mut object_namespace = namespaces::Level::new();  // TODO Keywords can't be in object namespace either

    let reverse_mapped_calls = transpiler_context.monomorphizer.get_mono_call_to_original_call();

    // Build ast
    for implementation in transpiler_context.exported_functions.iter() {
        // TODO Register with priority over internal symbols
        let ptr = &runtime.source.fn_pointers[reverse_mapped_calls.get(&implementation.head).unwrap_or(&implementation.head)];
        file_namespace.register_definition(implementation.head.function_id, &ptr.name);
    }

    for implementation in transpiler_context.internal_functions.iter() {
        let ptr = &runtime.source.fn_pointers[reverse_mapped_calls.get(&implementation.head).unwrap_or(&implementation.head)];
        // TODO Use underscore names?
        file_namespace.register_definition(implementation.head.function_id, &ptr.name);
    }

    for implementation in transpiler_context.exported_functions.iter().chain(transpiler_context.internal_functions.iter()) {
        let function_namespace = file_namespace.add_sublevel();
        for (variable, name) in implementation.variable_names.iter() {
            function_namespace.register_definition(variable.id.clone(), name);
        }
        for (expression_id, operation) in implementation.expression_forest.operations.iter() {
            if let ExpressionOperation::FunctionCall(fun) = operation {
                if let Some(BuiltinFunctionHint::Constructor) = runtime.source.fn_builtin_hints.get(&fun.function) {
                    let type_ = implementation.type_forest.resolve_binding_alias(expression_id).unwrap();
                    if let Entry::Vacant(entry) = struct_ids.entry(type_.clone()) {
                        // TODO If we have generics, we should include their bindings in the name somehow.
                        //  Eg. ArrayFloat. Probably only if it's exactly one. Otherwise, we need to be ok with
                        //  just the auto-renames.
                        let name = match &type_.unit {
                            TypeUnit::Struct(struct_) => &struct_.name,
                            // Technically only the name is unsupported here, but later we'd need to actually construct it too.
                            _ => panic!("Unsupported Constructor Type")
                        };
                        let id = Uuid::new_v4();
                        entry.insert(id);
                        // TODO Find proper names
                        file_namespace.register_definition(id, name);
                    }
                }
            }
        }
    }

    let mut names = global_namespace.map_names();
    names.extend(object_namespace.map_names());

    if module.main_functions.len() > 1 {
        return Err(InterpreterError::RuntimeError { msg: format!("Too many @main functions declared: {:?}", module.main_functions) });
    }

    let mut module = Box::new(ast::Module {
        // TODO Only classes used in the interface of exported functions are exported.
        //  Everything else is an internal class.
        exported_classes: vec![],
        exported_functions: vec![],
        internal_functions: vec![],
        main_function: module.main_functions.iter().exactly_one().ok()
            .map(|head| names[&head.function_id].clone())
    });

    for (struct_type, id) in struct_ids.iter() {
        if builtin_structs.contains(struct_type) {
            continue
        }

        let context = ClassContext {
            names: &names,
            struct_ids: &struct_ids,
            runtime,
        };

        module.exported_classes.push(transpile_class(struct_type, &context));
    }

    for (ref_, implementations) in [
        (&mut module.exported_functions, &transpiler_context.exported_functions),
        (&mut module.internal_functions, &transpiler_context.internal_functions),
    ] {
        for implementation in implementations.iter() {
            let context = FunctionContext {
                names: &names,
                expressions: &implementation.expression_forest,
                types: &implementation.type_forest,
                struct_ids: &struct_ids,
                runtime,
                fn_transpilation_hints: &transpiler_context.fn_transpilation_hints,
            };

            ref_.push(transpile_function(implementation, &context));
        }
    }

    Ok(module)
}
