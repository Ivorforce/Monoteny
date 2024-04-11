#[cfg(test)]
mod tests {
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
        test_transpiles("
    use!(module!(\"common\"));

    def main! :: {
        write_line(\"Hello World!\");
    };

    def transpile! :: {
        transpiler.add(main);
    };
")?;
        Ok(())
    }

    /// This tests generics, algebra and printing.
    #[test]
    fn simple_math() -> RResult<()> {
        test_transpiles("
precedence_order!([
    LeftUnaryPrecedence(LeftUnary),
    ExponentiationPrecedence(Right),
    MultiplicationPrecedence(Left),
    AdditionPrecedence(Left),
    ComparisonPrecedence(LeftConjunctivePairs),
    LogicalConjunctionPrecedence(Left),
    LogicalDisjunctionPrecedence(Left),
]);

![pattern(lhs + rhs, AdditionPrecedence)]
def _add(lhs '$Number, rhs '$Number) -> $Number :: add(lhs, rhs);

def main! :: {
    let a 'Float32 = 1 + 2;
};

def transpile! :: {
    transpiler.add(main);
};
")?;
        Ok(())
    }

    /// Tests if a static function created for a trait fulfillment (Eq) can be called.
    #[test]
    fn eq0() -> RResult<()> {
        test_transpiles("
def main! :: {
    is_equal(1, 1 'Int32);
};

def transpile! :: {
    transpiler.add(main);
};
")?;
        Ok(())
    }

    /// Tests if a function can call a requirements' function.
    #[test]
    fn eq1() -> RResult<()> {
        test_transpiles("
def is_equal_1(lhs '$Eq, rhs '$Eq) -> Bool :: is_equal(lhs, rhs);

def main! :: {
    is_equal_1(1, 1 'Int32);
};

def transpile! :: {
    transpiler.add(main);
};
")?;
        Ok(())
    }

    /// Tests if a function can call another function, passing its requirements fulfillment down.
    #[test]
    fn eq2() -> RResult<()> {
        test_transpiles("
def is_equal_1(lhs '$Eq, rhs '$Eq) -> Bool :: is_equal(lhs, rhs);
def is_equal_2(lhs '$Eq, rhs '$Eq) -> Bool :: is_equal_1(lhs, rhs);

def main! :: {
    is_equal_2(1, 1 'Int32);
};

def transpile! :: {
    transpiler.add(main);
};
")?;
        Ok(())
    }

    /// Tests whether monomorphization can yield two separate functions.
    #[test]
    fn monomorphize_branch() -> RResult<()> {
        let py_file = test_transpiles("
def (self '$Number).square() -> $Number :: multiply(self, self);

def main! :: {
    _write_line(\"\\(3.square() 'Int32)\");
    _write_line(\"\\(3.square() 'Float32)\");
};

def transpile! :: {
    transpiler.add(main);
};
")?;
        assert_eq!(py_file.match_indices("square").count(), 4);

        Ok(())
    }
}
