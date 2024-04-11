#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use itertools::Itertools;
    use crate::{interpreter, parser, transpiler};
    use crate::error::RResult;
    use crate::interpreter::Runtime;
    use crate::program::module::module_name;
    use crate::transpiler::LanguageContext;

    fn test_transpiles(code: &str) -> RResult<()> {
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

        Ok(())
    }

    /// This tests generics, algebra and printing.
    #[test]
    fn simple_math() -> RResult<()> {
        test_transpiles("
use!(module!(\"common\"));

def main! :: {
    write_line(1 + 2 'Float32);
};

def transpile! :: {
    transpiler.add(main);
};
")
    }

    /// Tests if a static function created for a trait fulfillment (Eq) can be called.
    #[test]
    fn eq0() -> RResult<()> {
        test_transpiles("
use!(module!(\"common\"));

def main! :: {
    is_equal(1, 1 'Int32);
};

def transpile! :: {
    transpiler.add(main);
};
")
    }

    /// Tests if a function can call a requirements' function.
    #[test]
    fn eq1() -> RResult<()> {
        test_transpiles("
use!(module!(\"common\"));

def is_equal_1(lhs '$Eq, rhs '$Eq) -> Bool :: is_equal(lhs, rhs);

def main! :: {
    is_equal_1(1, 1 'Int32);
};

def transpile! :: {
    transpiler.add(main);
};
")
    }

    /// Tests if a function can call another function, passing its requirements fulfillment down.
    #[test]
    fn eq2() -> RResult<()> {
        test_transpiles("
use!(module!(\"common\"));

def is_equal_1(lhs '$Eq, rhs '$Eq) -> Bool :: is_equal(lhs, rhs);
def is_equal_2(lhs '$Eq, rhs '$Eq) -> Bool :: is_equal_1(lhs, rhs);

def main! :: {
    is_equal_2(1, 1 'Int32);
};

def transpile! :: {
    transpiler.add(main);
};
")
    }
}
