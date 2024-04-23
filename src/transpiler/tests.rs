#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::rc::Rc;
    use itertools::Itertools;

    use crate::{interpreter, parser, transpiler};
    use crate::error::{RResult, RuntimeError};
    use crate::interpreter::run::gather_functions_logic;
    use crate::interpreter::Runtime;
    use crate::program::module::module_name;
    use crate::transpiler::{LanguageContext, Transpiler};

    fn test_transpiles(code: &str) -> RResult<String> {
        let (parsed, _) = parser::parse_program(code)?;

        let mut runtime = Runtime::new()?;
        runtime.repository.add("common", PathBuf::from("monoteny"));

        let module = runtime.load_ast_as_module(&parsed, module_name("main"))?;
        let context = transpiler::python::Context::new(&runtime);

        let transpiler = interpreter::run::transpile(&module, &mut runtime)?;
        let file_map = transpiler::transpile(transpiler, &mut runtime, &context, &transpiler::Config::default(), "main")?;

        let python_string = file_map["main.py"].to_string();
        assert!(python_string.contains("def main():"));

        Ok(python_string)
    }

    #[test]
    fn uninterpreted_hello_world() -> RResult<()> {
        let code = fs::read_to_string("test-code/hello_world.monoteny").unwrap();

        let (parsed, _) = parser::parse_program(code.as_str())?;

        let mut runtime = Runtime::new()?;
        runtime.repository.add("common", PathBuf::from("monoteny"));

        let module = runtime.load_ast_as_module(&parsed, module_name("main"))?;
        let main_function = module.main_functions.iter().at_most_one().unwrap().unwrap();

        let transpiler = Box::new(Transpiler {
            main_function: Some(Rc::clone(main_function)),
            exported_artifacts: gather_functions_logic(&runtime, &vec![main_function.function_id]),
        });

        let context = transpiler::python::Context::new(&runtime);
        let file_map = transpiler::transpile(transpiler, &mut runtime, &context, &transpiler::Config::default(), "main")?;

        let python_string = file_map["main.py"].to_string();
        assert!(python_string.contains("def main():"));

        Ok(())
    }

    #[test]
    fn hello_world() -> RResult<()> {
        test_transpiles(fs::read_to_string("test-code/hello_world.monoteny").unwrap().as_str())?;
        Ok(())
    }

    /// This tests generics, algebra and printing.
    #[test]
    fn custom_grammar() -> RResult<()> {
        test_transpiles(fs::read_to_string("test-code/grammar/custom_grammar.monoteny").unwrap().as_str())?;
        Ok(())
    }

    /// Tests if a static function created for a trait fulfillment (Eq) can be called.
    #[test]
    fn eq0() -> RResult<()> {
        test_transpiles(fs::read_to_string("test-code/requirements/eq0.monoteny").unwrap().as_str())?;
        Ok(())
    }

    /// Tests if a function can call a requirements' function.
    #[test]
    fn eq1() -> RResult<()> {
        test_transpiles(fs::read_to_string("test-code/requirements/eq1.monoteny").unwrap().as_str())?;
        Ok(())
    }

    /// Tests if a function can call another function, passing its requirements fulfillment down.
    #[test]
    fn eq2() -> RResult<()> {
        test_transpiles(fs::read_to_string("test-code/requirements/eq2.monoteny").unwrap().as_str())?;
        Ok(())
    }

    #[test]
    fn monomorphize_branch() -> RResult<()> {
        let py_file = test_transpiles(fs::read_to_string("test-code/monomorphization/branch.monoteny").unwrap().as_str())?;
        assert_eq!(py_file.match_indices("square").count(), 4);

        Ok(())
    }

    #[test]
    fn trait_conformance() -> RResult<()> {
        let py_file = test_transpiles(fs::read_to_string("test-code/traits/conformance.monoteny").unwrap().as_str())?;

        Ok(())
    }

    #[test]
    fn trait_fields() -> RResult<()> {
        let py_file = test_transpiles(fs::read_to_string("test-code/traits/fields.monoteny").unwrap().as_str())?;

        Ok(())
    }
}
