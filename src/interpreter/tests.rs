// #[cfg(test)]
// mod tests {
//     use std::fs;
//     use std::path::PathBuf;
//     use itertools::Itertools;
//     use crate::{interpreter, parser, transpiler};
//     use crate::error::RResult;
//     use crate::interpreter::Runtime;
//     use crate::parser::ast::*;
//     use crate::program::module::module_name;
//     use crate::transpiler::LanguageContext;
//
//     // FIXME Interpreter is broken right now.
//     /// This tests the transpiler, interpreter and function calls.
//     #[test]
//     fn run_hello_world() -> RResult<()> {
//         let mut runtime = Runtime::new()?;
//         runtime.repository.add("common", PathBuf::from("monoteny"));
//
//         let module = runtime.load_code(
//             fs::read_to_string("test-code/hello_world.monoteny").unwrap().as_str(),
//             module_name("main")
//         )?;
//
//         assert_eq!(module.exposed_functions.len(), 2);
//
//         // TODO Pass a pipe and monitor that "Hello World!" is printed.
//         interpreter::run::main(&module, &mut runtime)?;
//
//         Ok(())
//     }
// }
