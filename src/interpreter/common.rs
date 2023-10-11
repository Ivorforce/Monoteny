use std::path::PathBuf;
use std::rc::Rc;
use crate::interpreter::{RuntimeError, Runtime};

pub fn load(runtime: &mut Runtime) -> Result<(), Vec<RuntimeError>> {
    for name in [
        "precedence", "patterns", "math", "strings", "debug",
    ] {
        let module = runtime.load_file(&PathBuf::from(format!("monoteny/common/{}.monoteny", name)))?;
        runtime.source.module_order.push(name.to_string());
        runtime.source.module_by_name.insert(name.to_string(), module);
    }

    for ptr in runtime.source.module_by_name["debug"].fn_pointers.values() {
        runtime.function_evaluators.insert(ptr.target.unwrap_id(), match ptr.name.as_str() {
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

    Ok(())
}
