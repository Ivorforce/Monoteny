#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::rc::Rc;
    use itertools::Itertools;

    use crate::{interpreter, parser, transpiler};
    use crate::error::{RResult, RuntimeError};
    use crate::interpreter::run::gather_functions_logic;
    use crate::interpreter::runtime::Runtime;
    use crate::program::module::module_name;
    use crate::transpiler::{LanguageContext, Transpiler};

    fn test_transpiles(path: &str) -> RResult<String> {
        let mut runtime = Runtime::new()?;
        runtime.repository.add("common", PathBuf::from("monoteny"));

        let module = runtime.load_file_as_module(&PathBuf::from(path), module_name("main"))?;
        let context = transpiler::python::Context::new(&runtime);

        let transpiler = interpreter::run::transpile(&module, &mut runtime)?;
        let file_map = transpiler::transpile(transpiler, &mut runtime, &context, &transpiler::Config::default(), "main")?;

        let python_string = file_map["main.py"].to_string();
        assert!(python_string.contains("def main():"));

        Ok(python_string)
    }

    #[test]
    fn uninterpreted_hello_world() -> RResult<()> {
        let mut runtime = Runtime::new()?;
        runtime.repository.add("common", PathBuf::from("monoteny"));

        let module = runtime.load_file_as_module(&PathBuf::from("test-code/hello_world.monoteny"), module_name("main"))?;
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
        test_transpiles("test-code/hello_world.monoteny")?;
        Ok(())
    }

    /// This tests generics, algebra and printing.
    #[test]
    fn custom_grammar() -> RResult<()> {
        test_transpiles("test-code/grammar/custom_grammar.monoteny")?;
        Ok(())
    }

    /// Tests if a static function created for a trait fulfillment (Eq) can be called.
    #[test]
    fn eq0() -> RResult<()> {
        test_transpiles("test-code/requirements/eq0.monoteny")?;
        Ok(())
    }

    /// Tests if a function can call a requirements' function.
    #[test]
    fn eq1() -> RResult<()> {
        test_transpiles("test-code/requirements/eq1.monoteny")?;
        Ok(())
    }

    /// Tests if a function can call another function, passing its requirements fulfillment down.
    #[test]
    fn eq2() -> RResult<()> {
        test_transpiles("test-code/requirements/eq2.monoteny")?;
        Ok(())
    }

    #[test]
    fn monomorphize_branch() -> RResult<()> {
        let py_file = test_transpiles("test-code/monomorphization/branch.monoteny")?;
        assert_eq!(py_file.match_indices("square").count(), 4);

        Ok(())
    }

    #[test]
    fn trait_conformance() -> RResult<()> {
        let py_file = test_transpiles("test-code/traits/conformance.monoteny")?;

        Ok(())
    }

    #[test]
    fn trait_fields() -> RResult<()> {
        let py_file = test_transpiles("test-code/traits/fields.monoteny")?;

        Ok(())
    }

    #[test]
    fn string_interpolation() -> RResult<()> {
        let py_file = test_transpiles("test-code/grammar/string_interpolation.monoteny")?;

        Ok(())
    }
}
