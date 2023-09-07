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
use itertools::Itertools;
use uuid::Uuid;
use regex;
use crate::generic_unfolding::FunctionUnfolder;
use crate::interpreter;

use crate::program::builtins::Builtins;
use crate::program::computation_tree::*;
use crate::program::{find_annotated, Program};
use crate::program::calls::FunctionBinding;
use crate::program::global::{BuiltinFunctionHint, FunctionImplementation};
use crate::program::traits::RequirementsFulfillment;
use crate::program::types::TypeUnit;
use crate::transpiler::namespaces;
use crate::transpiler::python::class::{ClassContext, transpile_class};
use crate::transpiler::python::imperative::{FunctionContext, transpile_function};
use crate::transpiler::python::optimization::TranspilationHint;


pub fn transpile_program(program: &Program, builtins: &Rc<Builtins>) -> Box<ast::Module> {
    let mut struct_ids = HashMap::new();

    let mut global_namespace = builtins::create(&builtins, &mut struct_ids);
    let builtin_structs: HashSet<_> = struct_ids.keys().map(Clone::clone).collect();
    let mut file_namespace = global_namespace.add_sublevel();
    let mut object_namespace = namespaces::Level::new();  // TODO Keywords can't be in object namespace either

    let mut functions_by_id = HashMap::new();
    let mut builtin_hints_by_id = HashMap::new();
    let mut transpilation_hints_by_id = optimization::prepare(&builtins);
    let mut pointer_by_id = HashMap::new();

    for module in [&program.module].into_iter().chain(builtins.all_modules()) {
        for implementation in module.function_implementations.values() {
            functions_by_id.insert(implementation.implementation_id, Rc::clone(implementation));
        }
        for (head, hint) in module.builtin_hints.iter() {
            builtin_hints_by_id.insert(head.function_id, hint.clone());
        }
        for pointer in module.function_pointers.values() {
            pointer_by_id.insert(pointer.target.function_id, Rc::clone(pointer));
        }
    }

    let exported_symbols: Rc<RefCell<Vec<Rc<FunctionImplementation>>>> = Rc::new(RefCell::new(vec![]));
    let unfolder: Rc<RefCell<FunctionUnfolder>> = Rc::new(RefCell::new(FunctionUnfolder::new()));

    fn should_unfold(f: &Rc<FunctionBinding>, primitives: &HashMap<Uuid, BuiltinFunctionHint>, transpilation_hints_by_id: &HashMap<Uuid, TranspilationHint>) -> bool {
        if primitives.contains_key(&f.function.function_id) {
            // We need to inject these
            return false;
        }

        if transpilation_hints_by_id.contains_key(&f.function.function_id) {
            // We want to inject / override these
            return false;
        }

        true
    }

    // Run interpreter

    interpreter::run::transpile(program, &Rc::clone(&builtins), &|implementation| {
        if implementation.head.interface.collect_generics().len() > 0 {
            // We'll need to somehow transpile requirements as protocols and generics as generics.
            // That's for later!
            panic!("Transpiling generic functions is not supported yet: {:?}", implementation.head);
        }

        let unfolded_function = unfolder.borrow_mut().deref_mut().unfold_anonymous(
            implementation,
            &Rc::new(FunctionBinding {
                // The implementation's pointer is fine.
                function: Rc::clone(&implementation.head),
                // The resolution SHOULD be empty: The function is transpiled WITH its generics.
                // Unless generics are bound in the transpile directive, which is TODO
                requirements_fulfillment: RequirementsFulfillment::empty(),
            }),
            &|f| should_unfold(f, &builtin_hints_by_id, &transpilation_hints_by_id)
        );

        exported_symbols.borrow_mut().deref_mut().push(unfolded_function);
    });

    // Find and unfold internal symbols
    let mut exported_symbols_ = exported_symbols.borrow_mut();
    let exported_functions = exported_symbols_.deref_mut().clone();
    let mut unfolder_ = unfolder.borrow_mut();
    let unfolder = unfolder_.deref_mut();

    let mut internal_functions: Vec<Rc<FunctionImplementation>> = vec![];
    while let Some(used_symbol) = unfolder.new_mappable_calls.pop() {
        // TODO Use underscore names?
        let replacement_symbol = Rc::clone(&unfolder.mapped_calls[&used_symbol]);
        let implementation = &functions_by_id[&used_symbol.function.function_id];

        let unfolded_implementation = unfolder.unfold_anonymous(
            implementation,
            &replacement_symbol,
            &|f| should_unfold(f, &builtin_hints_by_id, &transpilation_hints_by_id)
        );

        internal_functions.push(unfolded_implementation);
    }

    let reverse_mapped_calls = unfolder.get_reverse_mapped_calls();

    for implementation in exported_functions.iter() {
        // TODO Register with priority over internal symbols
        let ptr = &pointer_by_id[&reverse_mapped_calls.get(&implementation.head).unwrap_or(&implementation.head).function_id];
        file_namespace.register_definition(implementation.head.function_id, &ptr.name);
    }

    for implementation in internal_functions.iter() {
        let ptr = &pointer_by_id[&reverse_mapped_calls.get(&implementation.head).unwrap_or(&implementation.head).function_id];
        file_namespace.register_definition(implementation.head.function_id, &ptr.name);
    }

    for implementation in exported_functions.iter().chain(internal_functions.iter()) {
        let function_namespace = file_namespace.add_sublevel();
        for (variable, name) in implementation.variable_names.iter() {
            function_namespace.register_definition(variable.id.clone(), name);
        }
        for (expression_id, operation) in implementation.expression_forest.operations.iter() {
            if let ExpressionOperation::FunctionCall(fun) = operation {
                if let Some(BuiltinFunctionHint::Constructor) = builtin_hints_by_id.get(&fun.function.function_id) {
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

    let mut module = Box::new(ast::Module {
        exported_classes: vec![],
        exported_functions: vec![],
        internal_functions: vec![],
        main_function: find_annotated(exported_functions.iter(), "main")
            .map(|main_function| names.get(&main_function.head.function_id).unwrap().clone()),
    });

    for (struct_type, id) in struct_ids.iter() {
        if builtin_structs.contains(struct_type) {
            continue
        }

        let context = ClassContext {
            names: &names,
            functions_by_id: &functions_by_id,
            builtins: &builtins,
            builtin_hints: &builtin_hints_by_id,
            struct_ids: &struct_ids,
        };

        module.exported_classes.push(transpile_class(struct_type, &context));
    }

    for (ref_, implementations) in [
        (&mut module.exported_functions, exported_functions),
        (&mut module.internal_functions, internal_functions),
    ] {
        for implementation in implementations.iter() {
            let context = FunctionContext {
                names: &names,
                functions_by_id: &functions_by_id,
                builtins: &builtins,
                builtin_hints: &builtin_hints_by_id,
                transpilation_hints: &transpilation_hints_by_id,
                expressions: &implementation.expression_forest,
                types: &implementation.type_forest,
                struct_ids: &struct_ids,
            };

            ref_.push(transpile_function(implementation, &context));
        }
    }

    module
}
