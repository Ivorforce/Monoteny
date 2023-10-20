pub mod types;
pub mod builtins;
pub mod class;
pub mod ast;
pub mod imperative;
pub mod representations;
pub mod keywords;

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::ops::DerefMut;
use std::rc::Rc;
use itertools::Itertools;
use linked_hash_map::LinkedHashMap;
use crate::error::RResult;
use crate::transpiler;
use crate::interpreter::Runtime;
use crate::program::function_object::FunctionRepresentation;

use crate::program::functions::FunctionHead;
use crate::program::global::{FunctionImplementation, FunctionLogic, FunctionLogicDescriptor};
use crate::refactor::simplify::Simplify;
use crate::refactor::Refactor;
use crate::transpiler::{Config, namespaces, structs, TranspiledArtifact, Transpiler};
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
    fn new(runtime: &Runtime) -> Self {
        let mut context = Context {
            representations: Representations::new(),
            builtin_global_namespace: namespaces::Level::new(),
            builtin_member_namespace: namespaces::Level::new(),
        };
        builtins::register_global(runtime, &mut context);
        context
    }

    fn make_files(&self, filename: &str, runtime: &mut Runtime, transpiler: Box<Transpiler>, config: &Config) -> RResult<HashMap<String, String>> {
        let mut refactor = Refactor::new(runtime);

        for artifact in transpiler.exported_artifacts {
            match artifact {
                TranspiledArtifact::Function(implementation) => {
                    let representation = refactor.runtime.source.fn_representations[&implementation.head].clone();
                    refactor.add(implementation, representation);
                }
            }
        }

        if config.should_monomorphize {
            let builtin_functions = self.representations.builtin_functions.clone();
            for head in refactor.explicit_functions.iter().cloned().collect_vec() {
                _ = refactor.monomorphize(head, &|binding| !builtin_functions.contains(&binding.function))
            }
        }
        else {
            todo!()
        }

        let mut simplify = Simplify::new(&mut refactor, config);
        simplify.run();

        // --- Reclaim from Refactor and make the ast

        let deep_calls = refactor.call_graph.deep_calls(refactor.explicit_functions.iter());
        let fn_representations = refactor.fn_representations;
        let mut fn_logic = refactor.fn_logic;

        let exported_functions = refactor.explicit_functions.iter()
            .map(|head| fn_logic.remove(head).unwrap().to_implementation())
            .try_collect()?;
        let mut implicit_functions = vec![];
        let mut internal_functions = HashMap::new();

        for head in deep_calls {
            // If it's an implementation, Refactor probably has it (although refactoring may have inlined to descriptors).
            match fn_logic.remove(&head).unwrap() {
                FunctionLogic::Implementation(i) => {
                    implicit_functions.push(i);
                }
                FunctionLogic::Descriptor(d) => {
                    internal_functions.insert(head, d);
                }
            }
        }

        let ast = create_ast(transpiler.main_function, &exported_functions, &implicit_functions, &internal_functions, &fn_representations, self)?;

        Ok(HashMap::from([
            (format!("{}.py", filename), ast.to_string())
        ]))
    }
}

pub fn create_ast(
    main_function: Option<Rc<FunctionHead>>,
    explicit_functions: &Vec<Box<FunctionImplementation>>,
    implicit_functions: &Vec<Box<FunctionImplementation>>,
    internal_functions: &HashMap<Rc<FunctionHead>, FunctionLogicDescriptor>,
    fn_representations: &HashMap<Rc<FunctionHead>, FunctionRepresentation>,
    context: &Context
) -> RResult<Box<ast::Module>> {
    let mut representations = context.representations.clone();
    let builtin_structs: HashSet<_> = representations.type_ids.keys().cloned().collect();

    let mut global_namespace = context.builtin_global_namespace.clone();
    // TODO We COULD have one namespace per object.
    //  But then we'll also need to register names on an object per object basis,
    //  and currently we have no way of identifying object namespaces easily.
    //  Maybe it will naturally arise later.
    let mut member_namespace = context.builtin_global_namespace.clone();
    let mut exports_namespace = global_namespace.add_sublevel();

    // ================= Names ==================

    // Names for exported functions
    for implementation in explicit_functions.iter() {
        representations::find_for_function(
            &mut representations.function_forms,
            &mut exports_namespace,
            implementation,
            &fn_representations[&implementation.head]
        )
    }

    // Names for exported structs
    let mut structs = LinkedHashMap::new();
    // TODO We only need to export structs that are mentioned in the interfaces of exported functions.
    //  But the following function doesn't work for now.
    // structs::find_in_interfaces(explicit_functions.iter().map(|i| &i.head), &mut structs);
    structs::find_in_implementations(explicit_functions, internal_functions, &mut structs);
    let exported_structs = structs.keys().cloned().collect_vec();
    for struct_ in structs.values() {
        exports_namespace.insert_name(struct_.trait_.id, struct_.trait_.name.as_str());
    }

    let mut internals_namespace = exports_namespace.add_sublevel();

    // We only really know from encountered calls which structs are left after monomorphization.
    // So let's just search the encountered calls.
    for implementation in explicit_functions.iter().chain(implicit_functions.iter()) {
        let function_namespace = internals_namespace.add_sublevel();
        // Map internal variable names
        for (ref_, name) in implementation.locals_names.iter() {
            function_namespace.insert_name(ref_.id, name);
        }
    }

    // Internal struct names
    structs::find_in_implementations(&implicit_functions, internal_functions, &mut structs);
    let internal_structs = structs.keys().filter(|s| !exported_structs.contains(s)).collect_vec();
    for type_ in internal_structs.iter() {
        let struct_ = &structs[*type_];
        internals_namespace.insert_name(struct_.trait_.id, struct_.trait_.name.as_str());
    }

    // Other struct pertaining functions
    for (type_, struct_) in structs.iter() {
        let namespace = member_namespace.add_sublevel();
        for (field, getter) in struct_.field_getters.iter() {
            let ptr = &fn_representations[getter];
            namespace.insert_name(field.id, ptr.name.as_str());
            representations.function_forms.insert(Rc::clone(getter), FunctionForm::GetMemberField(field.id));
        }
        for (field, getter) in struct_.field_setters.iter() {
            representations.function_forms.insert(Rc::clone(getter), FunctionForm::SetMemberField(field.id));
        }
        representations.function_forms.insert(Rc::clone(&struct_.constructor), FunctionForm::CallAsFunction);
        representations.type_ids.insert(type_.clone(), struct_.trait_.id);
    }

    // Internal / generated functions
    for implementation in implicit_functions.iter() {
        let representation = &fn_representations[&implementation.head];

        representations::find_for_function(
            &mut representations.function_forms,
            &mut internals_namespace,
            implementation, representation
        )
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

    for (type_, struct_) in structs.iter() {
        if builtin_structs.contains(type_) {
            continue
        }

        let context = ClassContext {
            names: &names,
            representations: &representations,
        };

        let statement = Box::new(Statement::Class(transpile_class(type_, &context)));
        let id = &representations.type_ids[type_];

        // TODO Only classes used in the interface of exported functions should be exported.
        //  Everything else is an internal class.
        module.exported_statements.push(statement);
        module.exported_names.insert(names[id].clone());
    }

    for (implementations, is_exported) in [
        (&explicit_functions, true),
        (&implicit_functions, false),
    ] {
        for implementation in implementations.iter() {
            let context = FunctionContext {
                names: &names,
                expressions: &implementation.expression_tree,
                types: &implementation.type_forest,
                representations: &representations,
                logic: internal_functions,
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
