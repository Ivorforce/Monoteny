use std::alloc::{alloc, Layout};
use std::path::PathBuf;
use std::rc::Rc;
use guard::guard;
use monoteny_macro::load_constant;
use uuid::Uuid;
use crate::error::RuntimeError;
use crate::interpreter::{Runtime, Value};
use crate::program::functions::FunctionHead;
use crate::program::types::TypeUnit;

pub fn load(runtime: &mut Runtime) -> Result<(), Vec<RuntimeError>> {
    for name in [
        "precedence", "patterns", "bool", "math", "strings", "debug", "transpilation",
    ] {
        let module = runtime.load_file(&PathBuf::from(format!("monoteny/common/{}.monoteny", name)))?;
        runtime.source.module_order.push(name.to_string());
        runtime.source.module_by_name.insert(name.to_string(), module);
    }

    for (function, representation) in runtime.source.module_by_name["debug"].fn_representations.iter() {
        runtime.function_evaluators.insert(function.unwrap_id(), match representation.name.as_str() {
            "_write_line" => Rc::new(move |interpreter, expression_id, binding| {{
                unsafe {{
                    let args = interpreter.implementation.expression_forest.arguments[&expression_id].clone();
                    let arg = interpreter.evaluate(args[0]).unwrap();
                    println!("{}", *(arg.data as *const String));

                    None
                }}
            }}),
            "_exit_with_error" => Rc::new(move |interpreter, expression_id, binding| {{
                unsafe {{
                    let args = interpreter.implementation.expression_forest.arguments[&expression_id].clone();
                    let arg = interpreter.evaluate(args[0]).unwrap();

                    panic!("{}", *(arg.data as *const String));
                }}
            }}),
            _ => continue,
        });
    }

    for (function, representation) in runtime.source.module_by_name["transpilation"].fn_representations.iter() {
        runtime.function_evaluators.insert(
            function.function_id,
            Rc::new(move |interpreter, expression_id, binding| {
                unsafe {
                    let arguments = interpreter.evaluate_arguments(expression_id);

                    // This may cause a SIGSEV if the callback pointer is invalidated. This should not happen as long as
                    //  nobody owns a Transpiler object outside of its lifetime.
                    let transpiler_callback = *(arguments[0].data as *const &dyn Fn(Rc<FunctionHead>, &Runtime));

                    let arg = &arguments[1];
                    let arg_id = &interpreter.implementation.expression_forest.arguments[&expression_id][1];
                    let arg_type = interpreter.implementation.type_forest.get_unit(arg_id).unwrap();

                    // TODO Once we have a Function supertype we can remove this check.
                    match arg_type {
                        TypeUnit::Function(f) => {},
                        _ => panic!("Argument to transpiler.add is not a function: {:?}", arg_type)
                    };

                    let implementation_id = *(arg.data as *const Uuid);
                    guard!(let implementation = &interpreter.runtime.source.fn_heads[&implementation_id] else {
                    panic!("Couldn't find function head: {}", implementation_id)
                });
                    transpiler_callback(Rc::clone(implementation), &interpreter.runtime);

                    return None;
                }
            })
        );
    }

    for (function, representation) in runtime.source.module_by_name["bool"].fn_representations.iter() {
        runtime.function_evaluators.insert(function.unwrap_id(), match representation.name.as_str() {
            "true" => load_constant!(bool true),
            "false" => load_constant!(bool false),
            _ => continue,
        });
    }

    Ok(())
}
