#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use guard::guard;
    use itertools::Itertools;
    use crate::{interpreter, parser, transpiler};
    use crate::error::RResult;
    use crate::interpreter::Runtime;
    use crate::parser::ast::*;
    use crate::program::module::module_name;
    use crate::transpiler::{Config, LanguageContext};

    const SIMPLE_PROGRAM: &str = "
use!(module!(\"common\"));

def main! :: {
    write_line(\"Hello World!\");
};

def transpile! :: {
    transpiler.add(main);
};
";

    #[test]
    fn parse() -> RResult<()> {
        let (parsed, errors) = parser::parse_program(SIMPLE_PROGRAM)?;
        assert!(errors.is_empty());

        assert_eq!(parsed.global_statements.len(), 3);

        guard!(let Statement::FunctionDeclaration(function) = &parsed.global_statements[1].as_ref().value else {
            panic!();
        });

        assert!(function.interface == FunctionInterface::Macro("main".to_string()));

        Ok(())
    }

    /// This tests the transpiler, interpreter and function calls.
    #[test]
    fn run() -> RResult<()> {
        let mut runtime = Runtime::new()?;
        runtime.repository.add("common", PathBuf::from("monoteny"));

        let module = runtime.load_code(SIMPLE_PROGRAM, module_name("main"))?;

        assert_eq!(module.exposed_functions.len(), 2);

        // TODO Pass a pipe and monitor that "Hello World!" is printed.
        interpreter::run::main(&module, &mut runtime)?;

        Ok(())
    }

    /// This tests generics, algebra and printing.
    #[test]
    fn transpile_math() -> RResult<()> {
        let (parsed, _) = parser::parse_program(&"
use!(module!(\"common\"));

def main! :: {
    write_line(1 + 2 'Float32);
};

def transpile! :: {
    transpiler.add(main);
};
".to_string())?;

        let mut runtime = Runtime::new()?;
        runtime.repository.add("common", PathBuf::from("monoteny"));

        let module = runtime.load_ast(&parsed, module_name("main"))?;
        let mut context = transpiler::python::create_context(&runtime);

        let transpiler = interpreter::run::transpile(&module, &mut runtime)?;

        let file_map = context.make_files("main", &runtime, transpiler, &Config {
            should_constant_fold: true,
            should_monomorphize: true,
        })?;
        let python_string = file_map["main.py"].to_string();
        assert!(python_string.contains("def main():"));

        Ok(())
    }
}
