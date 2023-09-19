#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use crate::{interpreter, parser, program, transpiler};
    use crate::interpreter::{common, Runtime};
    use crate::parser::ast::*;
    use crate::program::functions::ParameterKey;

    /// This tests the transpiler, interpreter and function calls.
    #[test]
    fn hello_world() {
        let parsed = parser::parse_program(&"
def @main :: {
    write_line(\"Hello World!\");
};

def @transpile :: {
    transpiler.add(main);
};
".to_string())?;
        assert_eq!(parsed.global_statements.len(), 2);
        assert!(parsed.global_statements[0].as_ref() == &GlobalStatement::FunctionDeclaration(Box::new(
            Function {
                target_type: None,
                identifier: "main".to_string(),
                parameters: vec![],
                decorators: vec!["main".to_string()],
                body: Some(Expression::from(vec![Box::new(Term::Block(vec![Box::new(Statement::Expression(Expression::from(vec![
                    Box::new(Term::Identifier("write_line".to_string())),
                    Box::new(Term::Struct(vec![
                        StructArgument {
                            key: ParameterKey::Positional,
                            value: Expression::from(vec![Box::new(Term::StringLiteral(vec![StringPart::Literal("Hello World!".to_string())]))]),
                            type_declaration: None,
                        }
                    ])),
                ])))]))])),
                return_type: None,
            }
        )));

        let builtins = program::builtins::create_builtins();
        let mut runtime = Runtime::new(&builtins);
        common::load(&mut runtime)?;

        let module = runtime.load_ast(&parsed)?;

        assert_eq!(module.function_implementations.len(), 2);
        let ptr = module.function_pointers.values().filter(|ptr| &ptr.name == "main").exactly_one().unwrap();
        let implementation = &module.function_implementations[&ptr.target];
        assert_eq!(implementation.expression_forest.operations.len(), 2);

        let python_ast = transpiler::python::transpile_module(&module, &mut runtime, true).expect("Python transpiler failed");
        let python_string = python_ast.to_string();
        assert!(python_string.contains("def main():"));
        assert!(python_string.contains("print(\"Hello World!\")"));
        assert!(python_string.contains("if __name__ == \"__main__\":"));
        assert!(!python_string.contains("transpile"));

        // TODO Pass a pipe and monitor that "Hello World!" is printed.
        interpreter::run::main(&module, &mut runtime).expect("Interpreter failed");
    }

    /// This tests generics, algebra and printing.
    #[test]
    fn math() {
        let parsed = parser::parse_program(&"
def @main :: {
    print(1 + 2 'Float32);
};

def @transpile :: {
    transpiler.add(main);
};
".to_string())?;

        let builtins = program::builtins::create_builtins();
        let mut runtime = Runtime::new(&builtins);
        common::load(&mut runtime)?;

        let module = runtime.load_ast(&parsed)?;

        let python_ast = transpiler::python::transpile_module(&module, &mut runtime, true).expect("Python transpiler failed");
        let python_string = python_ast.to_string();
        assert!(python_string.contains("def main():"));
    }
}
