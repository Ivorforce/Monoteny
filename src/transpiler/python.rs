pub mod types;
pub mod builtins;
pub mod class;
pub mod ast;
pub mod imperative;
pub mod representations;

use std::collections::HashSet;
use std::collections::hash_map::Entry;
use std::ops::DerefMut;
use itertools::Itertools;
use uuid::Uuid;
use crate::transpiler;
use crate::interpreter::{Runtime, InterpreterError};

use crate::program::computation_tree::*;
use crate::program::global::BuiltinFunctionHint;
use crate::program::module::Module;
use crate::program::types::TypeUnit;
use crate::transpiler::{constant_fold, namespaces, Transpiler};
use crate::transpiler::python::ast::Statement;
use crate::transpiler::python::class::{ClassContext, transpile_class};
use crate::transpiler::python::imperative::{FunctionContext, transpile_function};
use crate::transpiler::python::representations::Representations;


pub fn transpile_module(module: &Module, runtime: &mut Runtime, should_constant_fold: bool) -> Result<Box<ast::Module>, InterpreterError> {
    let mut representations = Representations::new();
    let builtin_namespace = builtins::register(runtime, &mut representations);

    // Run interpreter
    let mut transpiler = transpiler::run(module, runtime, transpiler::Context {
        builtin_functions: representations.builtin_functions.clone(),
    })?;

    if should_constant_fold {
        constant_fold(&mut transpiler);
    }

    // TODO We should sort the internal functions. This could be done roughly by putting them in the
    //  order the player defined it - which leaves only different monomorpizations to be sorted.
    //  Those can be sorted by something like the displayed 'function to string' (without randomized uuid).
    //  This should work because two traits sharing the same name but being different IDs should be rare.
    //  In that rare case, we can probably live with being indeterministic.

    create_ast(module, &transpiler, representations, runtime, builtin_namespace)
}

pub fn create_ast(module: &Module, transpiler_context: &Transpiler, mut representations: Representations, runtime: &Runtime, mut builtin_namespace: namespaces::Level) -> Result<Box<ast::Module>, InterpreterError> {
    let builtin_structs: HashSet<_> = representations.type_ids.keys().map(Clone::clone).collect();
    let mut file_namespace = builtin_namespace.add_sublevel();
    let mut object_namespace = namespaces::Level::new();  // TODO Actual keywords can't be in object namespace either

    let reverse_mapped_calls = transpiler_context.monomorphizer.get_mono_call_to_original_call();

    // ================= Names ==================

    for implementation in transpiler_context.exported_functions.iter() {
        // TODO Register with priority over internal symbols
        let ptr = &runtime.source.fn_pointers[reverse_mapped_calls.get(&implementation.head).unwrap_or(&implementation.head)];
        file_namespace.insert_name(implementation.head.function_id, &ptr.name);
    }

    for implementation in transpiler_context.internal_functions.iter() {
        let ptr = &runtime.source.fn_pointers[reverse_mapped_calls.get(&implementation.head).unwrap_or(&implementation.head)];
        // TODO Use underscore names?
        file_namespace.insert_name(implementation.head.function_id, &ptr.name);
    }

    for implementation in transpiler_context.exported_functions.iter().chain(transpiler_context.internal_functions.iter()) {
        let function_namespace = file_namespace.add_sublevel();
        for (variable, name) in implementation.variable_names.iter() {
            function_namespace.insert_name(variable.id.clone(), name);
        }
        for (expression_id, operation) in implementation.expression_forest.operations.iter() {
            if let ExpressionOperation::FunctionCall(fun) = operation {
                if let Some(BuiltinFunctionHint::Constructor) = runtime.source.fn_builtin_hints.get(&fun.function) {
                    let type_ = implementation.type_forest.resolve_binding_alias(expression_id).unwrap();
                    if let Entry::Vacant(entry) = representations.type_ids.entry(type_.clone()) {
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
                        file_namespace.insert_name(id, name);
                    }
                }
            }
        }
    }

    let mut names = builtin_namespace.map_names();
    names.extend(object_namespace.map_names());

    if module.main_functions.len() > 1 {
        return Err(InterpreterError::RuntimeError { msg: format!("Too many @main functions declared: {:?}", module.main_functions) });
    }

    // ================= Representations ==================

    representations::find_for_functions(
        &mut representations.function_representations,
        &names,
        transpiler_context.exported_functions.iter().chain(transpiler_context.internal_functions.iter())
    );

    // ================= Build AST ==================

    let mut module = Box::new(ast::Module {
        exported_statements: vec![],
        internal_statements: vec![],
        exported_names: HashSet::new(),
        main_function: module.main_functions.iter().exactly_one().ok()
            .map(|head| names[&head.function_id].clone())
    });

    for (struct_type, id) in representations.type_ids.iter() {
        if builtin_structs.contains(struct_type) {
            continue
        }

        let context = ClassContext {
            names: &names,
            runtime,
            representations: &representations,
        };

        let statement = Box::new(Statement::Class(transpile_class(struct_type, &context)));

        // TODO Only classes used in the interface of exported functions should be exported.
        //  Everything else is an internal class.
        module.exported_statements.push(statement);
        module.exported_names.insert(context.names[id].clone());
    }

    for (implementations, is_exported) in [
        (&transpiler_context.exported_functions, true),
        (&transpiler_context.internal_functions, false),
    ] {
        for implementation in implementations.iter() {
            let context = FunctionContext {
                names: &names,
                expressions: &implementation.expression_forest,
                types: &implementation.type_forest,
                runtime,
                representations: &representations,
            };

            let transpiled = transpile_function(implementation, &context);

            if is_exported {
                module.exported_names.insert(context.names[&implementation.head.function_id].clone());
                module.exported_statements.push(transpiled);
            }
            else {
                module.internal_statements.push(transpiled);
            }
        }
    }

    Ok(module)
}
