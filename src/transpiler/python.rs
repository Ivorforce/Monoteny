use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use display_with_options::{IndentOptions, with_options};
use itertools::Itertools;
use linked_hash_map::LinkedHashMap;

use crate::error::RResult;
use crate::interpreter::runtime::Runtime;
use crate::program::functions::FunctionLogicDescriptor;
use crate::refactor::Refactor;
use crate::transpiler;
use crate::transpiler::{namespaces, structs, TranspilePackage};
use crate::transpiler::python::ast::Statement;
use crate::transpiler::python::class::{ClassContext, transpile_class};
use crate::transpiler::python::imperative::{FunctionContext, transpile_function};
use crate::transpiler::python::representations::{FunctionForm, Representations};

pub mod types;
pub mod builtins;
pub mod class;
pub mod ast;
pub mod imperative;
pub mod representations;
pub mod keywords;
mod strings;

pub struct Context {
    pub representations: Representations,
    pub builtin_global_namespace: namespaces::Level,
    pub builtin_member_namespace: namespaces::Level,
}

impl transpiler::LanguageContext for Context {
    fn new(runtime: &Runtime) -> Self {
        let mut context = Context {
            representations: Representations::new(),
            builtin_global_namespace: namespaces::Level::new(),
            builtin_member_namespace: namespaces::Level::new(),
        };
        builtins::register_global(runtime, &mut context);
        context
    }

    fn register_builtins(&self, refactor: &mut Refactor) {
        // TODO If there's any optimizations we know (e.g. sin()), place it here.
    }

    fn refactor_code(&self, refactor: &mut Refactor) {
        // TODO We need to at least break up inner blocks of all functions.
    }

    fn make_files(&self, base_filename: &str, package: TranspilePackage) -> RResult<HashMap<String, String>> {
        let ast = self.create_ast(package)?;

        let string = format!("{}", with_options(ast.as_ref(), &IndentOptions {
            full_indentation: String::new(),
            next_level: "    ",
        }));

        Ok(HashMap::from([
            (format!("{}.py", base_filename), string)
        ]))
    }
}

impl Context {
    pub fn create_ast(&self, transpile: TranspilePackage) -> RResult<Box<ast::Module>> {
        let mut representations = self.representations.clone();
        let builtin_structs: HashSet<_> = representations.type_ids.keys().cloned().collect();

        let mut global_namespace = self.builtin_global_namespace.clone();
        // TODO We COULD have one namespace per object.
        //  But then we'll also need to register names on an object per object basis,
        //  and currently we have no way of identifying object namespaces easily.
        //  Maybe it will naturally arise later.
        let mut member_namespace = self.builtin_global_namespace.clone();
        let mut exports_namespace = global_namespace.add_sublevel();

        // ================= Names ==================

        // Names for exported functions
        for implementation in transpile.explicit_functions.iter() {
            representations::find_for_function(
                &mut representations.function_forms,
                &mut exports_namespace,
                implementation,
            )
        }

        // Names for exported structs
        let mut structs = LinkedHashMap::new();
        // TODO We only need to export structs that are mentioned in the interfaces of exported functions.
        //  But the following function doesn't work for now.
        // structs::find_in_interfaces(explicit_functions.iter().map(|i| &i.head), &mut structs);
        structs::find_in_implementations(&transpile.explicit_functions, &transpile.used_native_functions, &mut structs);
        let exported_structs = structs.keys().cloned().collect_vec();
        for struct_ in structs.values() {
            exports_namespace.insert_name(struct_.trait_.id, struct_.trait_.name.as_str());
        }

        let mut internals_namespace = exports_namespace.add_sublevel();

        // We only really know from encountered calls which structs are left after monomorphization.
        // So let's just search the encountered calls.
        for implementation in transpile.explicit_functions.iter().chain(transpile.implicit_functions.iter()) {
            let function_namespace = internals_namespace.add_sublevel();
            // Map internal variable names
            for (ref_, name) in implementation.locals_names.iter() {
                function_namespace.insert_name(ref_.id, name);
            }
        }

        // Internal struct names
        structs::find_in_implementations(&transpile.implicit_functions, &transpile.used_native_functions, &mut structs);
        let internal_structs = structs.keys().filter(|s| !exported_structs.contains(s)).collect_vec();
        for type_ in internal_structs.iter() {
            let struct_ = &structs[*type_];
            internals_namespace.insert_name(struct_.trait_.id, struct_.trait_.name.as_str());
        }

        // Other struct pertaining functions
        for (type_, struct_) in structs.iter() {
            let namespace = member_namespace.add_sublevel();
            for (field, getter) in struct_.field_getters.iter() {
                namespace.insert_name(field.id, getter.declared_representation.name.as_str());
                representations.function_forms.insert(Rc::clone(getter), FunctionForm::GetMemberField(field.id));
            }
            for (field, getter) in struct_.field_setters.iter() {
                representations.function_forms.insert(Rc::clone(getter), FunctionForm::SetMemberField(field.id));
            }
            representations.function_forms.insert(Rc::clone(&struct_.constructor), FunctionForm::CallAsFunction);
            representations.type_ids.insert(type_.clone(), struct_.trait_.id);
        }

        // Internal / generated functions
        for implementation in transpile.implicit_functions.iter() {
            representations::find_for_function(
                &mut representations.function_forms,
                &mut internals_namespace,
                implementation,
            )
        }

        for (native_function, descriptor) in transpile.used_native_functions.iter() {
            match descriptor {
                FunctionLogicDescriptor::Stub => {}
                FunctionLogicDescriptor::Clone(_) => {}
                FunctionLogicDescriptor::TraitProvider(trait_) => {
                    representations.function_forms.insert(Rc::clone(&native_function), FunctionForm::Constant(trait_.id));
                }
                FunctionLogicDescriptor::FunctionProvider(_) => {}
                FunctionLogicDescriptor::PrimitiveOperation { .. } => {}
                FunctionLogicDescriptor::Constructor(_) => {}
                FunctionLogicDescriptor::GetMemberField(_, _) => {}
                FunctionLogicDescriptor::SetMemberField(_, _) => {}
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
            main_function: transpile.main_function.map(|head| names[&head.function_id].clone())
        });

        let mut unestablished_structs = structs.keys().map(Rc::clone).collect();
        for (type_, struct_) in structs.iter() {
            if builtin_structs.contains(type_) {
                continue
            }

            let context = ClassContext {
                names: &names,
                representations: &representations,
                unestablished_structs: &unestablished_structs,
            };

            let statement = Box::new(Statement::Class(transpile_class(type_, &context)));
            let id = &representations.type_ids[type_];

            // TODO Only classes used in the interface of exported functions should be exported.
            //  Everything else is an internal class.
            module.exported_statements.push(statement);
            module.exported_names.insert(names[id].clone());

            unestablished_structs.remove(type_);
        }

        for (implementations, is_exported) in [
            (&transpile.explicit_functions, true),
            (&transpile.implicit_functions, false),
        ] {
            for implementation in implementations.iter() {
                let context = FunctionContext {
                    names: &names,
                    expressions: &implementation.expression_tree,
                    types: &implementation.type_forest,
                    representations: &representations,
                    logic: &transpile.used_native_functions,
                };

                let transpiled = transpile_function(implementation, &context);

                if is_exported {
                    module.exported_names.insert(names[&implementation.head.function_id].clone());
                    module.exported_statements.push(transpiled);
                }
                else {
                    module.internal_statements.push(transpiled);
                }
            }
        }

        Ok(module)
    }
}
