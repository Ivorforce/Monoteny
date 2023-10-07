pub mod types;
pub mod builtins;
pub mod class;
pub mod ast;
pub mod imperative;
pub mod representations;
pub mod keywords;

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::ops::DerefMut;
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use uuid::Uuid;
use crate::error::{RResult, RuntimeError};
use crate::transpiler;
use crate::interpreter::Runtime;
use crate::program::computation_tree::ExpressionOperation;

use crate::program::functions::FunctionHead;
use crate::program::global::BuiltinFunctionHint;
use crate::program::types::TypeUnit;
use crate::transpiler::{namespaces, Transpiler};
use crate::transpiler::python::ast::Statement;
use crate::transpiler::python::class::{ClassContext, transpile_class};
use crate::transpiler::python::imperative::{FunctionContext, transpile_function};
use crate::transpiler::python::representations::{FunctionForm, Representations};


pub struct Context {
    pub representations: Representations,
    pub builtin_global_namespace: namespaces::Level,
    pub builtin_member_namespace: namespaces::Level,
}

impl transpiler::Context for Context {
    fn builtin_functions(&self) -> HashSet<Rc<FunctionHead>> {
        self.representations.builtin_functions.clone()
    }

    fn make_files(&self, filename: &str, runtime: &Runtime, transpiler: &Transpiler) -> Result<HashMap<String, String>, Vec<RuntimeError>> {
        let ast = create_ast(transpiler, self, runtime).map_err(|e| vec![e])?;

        Ok(HashMap::from([
            (format!("{}.py", filename), ast.to_string())
        ]))
    }
}

pub fn create_context(runtime: &Runtime) -> Context {
    let mut context = Context {
        representations: Representations::new(),
        builtin_global_namespace: namespaces::Level::new(),
        builtin_member_namespace: namespaces::Level::new(),
    };
    builtins::register_global(runtime, &mut context);
    context
}

pub fn create_ast(transpiler: &Transpiler, context: &Context, runtime: &Runtime) -> RResult<Box<ast::Module>> {
    let mut representations = context.representations.clone();
    let builtin_structs: HashSet<_> = representations.type_ids.keys().cloned().collect();

    let mut global_namespace = context.builtin_global_namespace.clone();
    // TODO We COULD have one namespace per object.
    //  But then we'll also need to register names on an object per object basis,
    //  and currently we have no way of identifying object namespaces easily.
    //  Maybe it will naturally arise later.
    let mut member_namespace = context.builtin_global_namespace.clone();
    let mut file_namespace = global_namespace.add_sublevel();

    let reverse_mapped_calls = transpiler.monomorphizer.get_mono_call_to_original_call();

    // ================= Names ==================

    for (trait_, reference) in runtime.source.trait_references.iter() {
        // TODO This should not be fixed - but it currently clashes otherwise with Constructor's name choosing.
        //  Technically the trait references should be monomorphized, because an access to Vec<String> is not the same
        //  after monomorphization as Vec<Int32>. They should be two different constants.
        file_namespace.insert_fixed_name(reference.id, trait_.name.as_str());
    }

    // We only really know from encountered calls which structs are left after monomorphization.
    // So let's just search the encountered calls.
    for implementation in transpiler.exported_functions.iter().chain(transpiler.internal_functions.iter()) {
        let function_namespace = file_namespace.add_sublevel();
        // Map internal variable names
        for (variable, name) in implementation.variable_names.iter() {
            function_namespace.insert_name(variable.id.clone(), name);
        }

        for (expression_id, operation) in implementation.expression_forest.operations.iter() {
            if let ExpressionOperation::FunctionCall(binding) = operation {
                guard!(let Some(hint) = runtime.source.fn_builtin_hints.get(&binding.function) else {
                    continue;
                });
                let hint: &BuiltinFunctionHint = hint;

                match hint {
                    BuiltinFunctionHint::Constructor(object_refs) => {
                        let type_ = &binding.function.interface.return_type;  // Fulfillment for Self
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
                            file_namespace.insert_fixed_name(id, name);
                            representations.function_representations.insert(
                                Rc::clone(&binding.function),
                                FunctionForm::CallAsFunction
                            );
                        }
                    }
                    BuiltinFunctionHint::Getter(ref_) => {
                        let ptr = &runtime.source.fn_pointers[&binding.function];
                        member_namespace.insert_fixed_name(ref_.id, &ptr.name);  // TODO This should not be fixed.
                        representations.function_representations.insert(
                            Rc::clone(&binding.function),
                            FunctionForm::GetMemberField(ref_.id)
                        );
                    }
                    BuiltinFunctionHint::Setter(ref_) => {
                        let ptr = &runtime.source.fn_pointers[&binding.function];
                        member_namespace.insert_fixed_name(ref_.id, &ptr.name);  // TODO This should not be fixed.
                        representations.function_representations.insert(
                            Rc::clone(&binding.function),
                            FunctionForm::SetMemberField(ref_.id)
                        );
                    }
                    _ => {},
                }
            }
        }
    }

    // ================= Representations ==================

    for implementation in transpiler.exported_functions.iter() {
        // TODO Register with priority over internal symbols. Internal functions can use understore prefix if need be.
        let ptr = &runtime.source.fn_pointers[reverse_mapped_calls.get(&implementation.head).unwrap_or(&implementation.head)];
        representations::find_for_function(
            &mut representations.function_representations,
            &mut file_namespace,
            implementation, ptr
        );
    }

    for implementation in transpiler.internal_functions.iter() {
        let ptr = &runtime.source.fn_pointers[reverse_mapped_calls.get(&implementation.head).unwrap_or(&implementation.head)];
        representations::find_for_function(
            &mut representations.function_representations,
            &mut file_namespace,
            implementation, ptr
        );
    }

    // ================= Build AST ==================

    // Finally, the names can be locked in.
    let mut names = global_namespace.map_names();
    names.extend(member_namespace.map_names());

    let mut module = Box::new(ast::Module {
        exported_statements: vec![],
        internal_statements: vec![],
        exported_names: HashSet::new(),
        main_function: transpiler.main_function.clone().map(|head| names[&head.function_id].clone())
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
        (&transpiler.exported_functions, true),
        (&transpiler.internal_functions, false),
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
