use std::path::PathBuf;
use std::rc::Rc;

use crate::error::RResult;
use crate::interpreter::chunks::OpCode;
use crate::interpreter::compiler::FunctionCompiler;
use crate::interpreter::Runtime;
use crate::program::global::FunctionLogicDescriptor;
use crate::program::module::module_name;

pub fn load(runtime: &mut Runtime) -> RResult<()> {
    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Monoteny files --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    runtime.repository.add("core", PathBuf::from("monoteny"));
    runtime.get_or_load_module(&module_name("core"))?;

    for function in runtime.source.module_by_name[&module_name("core.debug")].explicit_functions(&runtime.source) {
        let representation = &runtime.source.fn_representations[function];

        runtime.function_inlines.insert(Rc::clone(function), match representation.name.as_str() {
            "_write_line" => Rc::new(move |compiler| {{
                compiler.chunk.push(OpCode::PRINT);
            }}),
            "_exit_with_error" => Rc::new(move |compiler| {{
                panic!();
            }}),
            _ => continue,
        });
    }

    for function in runtime.source.module_by_name[&module_name("core.transpilation")].explicit_functions(&runtime.source) {
        let representation = &runtime.source.fn_representations[function];

        runtime.function_inlines.insert(Rc::clone(function), match representation.name.as_str() {
            "add" => Rc::new(move |compiler| {{
                compiler.chunk.push(OpCode::TRANSPILE_ADD);
            }}),
            _ => continue,
        });
    }

    Ok(())
}
