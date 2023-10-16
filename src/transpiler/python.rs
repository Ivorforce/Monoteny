pub mod types;
pub mod builtins;
pub mod class;
pub mod ast;
pub mod imperative;
pub mod representations;
pub mod keywords;

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::ops::DerefMut;
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use uuid::Uuid;
use crate::error::RResult;
use crate::transpiler;
use crate::interpreter::Runtime;
use crate::program::computation_tree::ExpressionOperation;

use crate::program::functions::FunctionHead;
use crate::program::global::{BuiltinFunctionHint, FunctionImplementation};
use crate::program::types::TypeUnit;
use crate::refactor::constant_folding::ConstantFold;
use crate::refactor::Refactor;
use crate::transpiler::{Config, namespaces, TranspiledArtifact, Transpiler};
use crate::transpiler::python::ast::Statement;
use crate::transpiler::python::class::{ClassContext, transpile_class};
use crate::transpiler::python::imperative::{FunctionContext, transpile_function};
use crate::transpiler::python::representations::{FunctionForm, Representations};


pub struct Context {
    pub representations: Representations,
    pub builtin_global_namespace: namespaces::Level,
    pub builtin_member_namespace: namespaces::Level,
}

impl transpiler::LanguageContext for Context {
    fn make_files(&self, filename: &str, runtime: &Runtime, transpiler: Box<Transpiler>, config: &Config) -> RResult<HashMap<String, String>> {
        let mut refactor = Refactor::new(runtime);

        for artifact in transpiler.exported_artifacts {
            match artifact {
                TranspiledArtifact::Function(implementation) => {
                    refactor.add(implementation);
                }
            }
        }

        if config.should_monomorphize {
            let builtin_functions = self.representations.builtin_functions.clone();
            for head in refactor.explicit_functions.iter().cloned().collect_vec() {
                refactor.monomorphize(head, &|binding| !builtin_functions.contains(&binding.function))
            }
        }
        else {
            todo!()
        }

        if config.should_constant_fold {
            let mut constant_folder = ConstantFold::new(&mut refactor);
            constant_folder.run();
        }

        let mapped_call_to_user_call = refactor.monomorphize.get_mono_call_to_original_call();
        let exported_functions = refactor.explicit_functions.iter().map(|head| refactor.implementation_by_head.remove(head).unwrap()).collect_vec();
        let internal_functions = refactor.invented_functions.iter().map(|head| refactor.implementation_by_head.remove(head).unwrap()).collect_vec();
        let ast = create_ast(transpiler.main_function, exported_functions, internal_functions, &mapped_call_to_user_call, self, runtime)?;

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

pub fn create_ast(main_function: Option<Rc<FunctionHead>>, exported_functions: Vec<Box<FunctionImplementation>>, internal_functions: Vec<Box<FunctionImplementation>>, mapped_call_to_user_call: &HashMap<Rc<FunctionHead>, Rc<FunctionHead>>, context: &Context, runtime: &Runtime) -> RResult<Box<ast::Module>> {
    let mut representations = context.representations.clone();
    let builtin_structs: HashSet<_> = representations.type_ids.keys().cloned().collect();

    let mut global_namespace = context.builtin_global_namespace.clone();
    // TODO We COULD have one namespace per object.
    //  But then we'll also need to register names on an object per object basis,
    //  and currently we have no way of identifying object namespaces easily.
    //  Maybe it will naturally arise later.
    let mut member_namespace = context.builtin_global_namespace.clone();
    let mut file_namespace = global_namespace.add_sublevel();

    // ================= Names ==================

    for (head, trait_) in runtime.source.trait_references.iter() {
        // TODO This should not be fixed - but it currently clashes otherwise with Constructor's name choosing.
        //  Technically the trait references should be monomorphized, because an access to Vec<String> is not the same
        //  after monomorphization as Vec<Int32>. They should be two different constants.
        file_namespace.insert_name(trait_.id, trait_.name.as_str());
        representations.function_representations.insert(Rc::clone(head), FunctionForm::Constant(trait_.id));
    }

    // We only really know from encountered calls which structs are left after monomorphization.
    // So let's just search the encountered calls.
    for implementation in exported_functions.iter().chain(internal_functions.iter()) {
        let function_namespace = file_namespace.add_sublevel();
        // Map internal variable names
        for (ref_, name) in implementation.locals_names.iter() {
            println!("Local {:?}", ref_);
            function_namespace.insert_name(ref_.id, name);
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
                            let (name, trait_id) = match &type_.unit {
                                TypeUnit::Struct(trait_) => (&trait_.name, trait_.id),
                                // Technically only the name is unsupported here, but later we'd need to actually construct it too.
                                _ => panic!("Unsupported Constructor Type")
                            };
                            // TODO This logic will fall apart if we have multiple instantiations of the same type.
                            //  In that case we probably want to monomorphize the struct getter per-object so we can
                            //  differentiate them and assign different names.
                            entry.insert(trait_id);
                            representations.function_representations.insert(
                                Rc::clone(&binding.function),
                                FunctionForm::CallAsFunction
                            );
                        }
                    }
                    BuiltinFunctionHint::GetMemberField(ref_) => {
                        let ptr = &runtime.source.fn_representations[&binding.function];
                        file_namespace.insert_name(ref_.id, &ptr.name);  // TODO We should run over all members of all used structs, not just the members we happen to use.
                        representations.function_representations.insert(
                            Rc::clone(&binding.function),
                            FunctionForm::GetMemberField(ref_.id)
                        );
                    }
                    BuiltinFunctionHint::SetMemberField(ref_) => {
                        let ptr = &runtime.source.fn_representations[&binding.function];
                        file_namespace.insert_name(ref_.id, &ptr.name);  // TODO We should run over all members of all used structs, not just the members we happen to use.
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

    for (implementations, is_exported) in [
        (&exported_functions, true),
        (&internal_functions, false),
    ] {
        for implementation in implementations.iter() {
            // TODO Register with priority over internal symbols. Internal functions can use understore prefix if need be.
            let representation = &runtime.source.fn_representations[mapped_call_to_user_call.get(&implementation.head).unwrap_or(&implementation.head)];
            representations::find_for_function(
                &mut representations.function_representations,
                &mut file_namespace,
                implementation, representation.clone()
            )
        }
    }

    // ================= Build AST ==================

    // Finally, the names can be locked in.
    let mut names = global_namespace.map_names();
    names.extend(member_namespace.map_names());

    let mut module = Box::new(ast::Module {
        exported_statements: vec![],
        internal_statements: vec![],
        exported_names: HashSet::new(),
        main_function: main_function.map(|head| names[&head.function_id].clone())
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
        (&exported_functions, true),
        (&internal_functions, false),
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
