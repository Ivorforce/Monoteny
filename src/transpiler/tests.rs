#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use itertools::Itertools;
    use crate::{interpreter, parser, transpiler};
    use crate::error::RResult;
    use crate::interpreter::Runtime;
    use crate::program::module::module_name;
    use crate::transpiler::LanguageContext;

    fn test_transpiles(code: &str) -> RResult<String> {
        let (parsed, _) = parser::parse_program(code)?;

        let mut runtime = Runtime::new()?;
        runtime.repository.add("common", PathBuf::from("monoteny"));

        let module = runtime.load_ast(&parsed, module_name("main"))?;
        let context = transpiler::python::Context::new(&runtime);

        let transpiler = interpreter::run::transpile(&module, &mut runtime)?;
        let file_map = transpiler::transpile(&module, &mut runtime, &context, &transpiler::Config {
            should_monomorphize: true,
            should_constant_fold: false,
            should_inline: false,
            should_trim_locals: false,
        }, "main")?;

        let python_string = file_map["main.py"].to_string();
        assert!(python_string.contains("def main():"));

        Ok(python_string)
    }

    #[test]
    fn hello_world() -> RResult<()> {
        test_transpiles(fs::read_to_string("test-code/hello_world.monoteny").unwrap().as_str())?;
        Ok(())
    }

    /// This tests generics, algebra and printing.
    #[test]
    fn custom_grammar() -> RResult<()> {
        test_transpiles(fs::read_to_string("test-code/custom_grammar.monoteny").unwrap().as_str())?;
        Ok(())
    }

    /// Tests if a static function created for a trait fulfillment (Eq) can be called.
    #[test]
    fn eq0() -> RResult<()> {
        test_transpiles(fs::read_to_string("test-code/eq0.monoteny").unwrap().as_str())?;
        Ok(())
    }

    /// Tests if a function can call a requirements' function.
    #[test]
    fn eq1() -> RResult<()> {
        test_transpiles(fs::read_to_string("test-code/eq1.monoteny").unwrap().as_str())?;
        Ok(())
    }

    /// Tests if a function can call another function, passing its requirements fulfillment down.
    #[test]
    fn eq2() -> RResult<()> {
        test_transpiles(fs::read_to_string("test-code/eq2.monoteny").unwrap().as_str())?;
        Ok(())
    }

    /// Tests whether monomorphization can yield two separate functions.
    #[test]
    fn monomorphize_branch() -> RResult<()> {
        let py_file = test_transpiles(fs::read_to_string("test-code/monomorphize_branch.monoteny").unwrap().as_str())?;
        assert_eq!(py_file.match_indices("square").count(), 4);

        Ok(())
    }
}
