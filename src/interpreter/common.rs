use std::path::PathBuf;
use std::rc::Rc;
use crate::interpreter::{InterpreterError, Runtime};

pub fn load(runtime: &mut Runtime) -> Result<(), InterpreterError> {
    for name in [
        "math", "debug", "strings",
    ] {
        let module = runtime.load_file(&PathBuf::from(format!("monoteny/common/{}.monoteny", name)))?;
        runtime.source.module_by_name.insert(name.into(), module);
    }

    for ptr in runtime.source.module_by_name["debug".into()].fn_pointers.values() {
        runtime.function_evaluators.insert(ptr.target.unwrap_id(), match ptr.name.as_str() {
            "_print" => Rc::new(move |interpreter, expression_id, binding| {{
                unsafe {{
                    let args = interpreter.implementation.expression_forest.arguments[&expression_id].clone();
                    let arg = interpreter.evaluate(args[0]).unwrap();
                    println!("{}", *(arg.data as *const String));

                    None
                }}
            }}),
            "panic" => Rc::new(move |interpreter, expression_id, binding| {{
                panic!()
            }}),
            "todo" => Rc::new(move |interpreter, expression_id, binding| {{
                todo!()
            }})
            ,
            _ => continue,
        });
    }

    Ok(())
}
