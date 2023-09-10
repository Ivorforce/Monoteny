#[cfg(test)]
mod tests {
    use std::io::BufWriter;
    use itertools::Itertools;
    use crate::{interpreter, linker, parser, program, transpiler};
    use crate::interpreter::InterpreterGlobals;
    use crate::parser::ast::*;
    use crate::program::functions::ParameterKey;

    /// This tests the transpiler, interpreter and function calls.
    #[test]
    fn hello_world() {
        let parsed = parser::parse_program(&"
@main
def main() :: {
    print(\"Hello World!\");
};

@transpile
def transpile(transpiler 'Transpiler) :: {
    transpiler.add(main);
};
".to_string());
        assert_eq!(parsed.global_statements.len(), 2);
        assert!(parsed.global_statements[0].as_ref() == &GlobalStatement::FunctionDeclaration(Box::new(
            Function {
                target_type: None,
                identifier: "main".to_string(),
                parameters: vec![],
                decorators: vec!["main".to_string()],
                body: Some(Expression::from(vec![Box::new(Term::Scope(vec![Box::new(Statement::Expression(Expression::from(vec![
                    Box::new(Term::Identifier("print".to_string())),
                    Box::new(Term::Struct(vec![
                        StructArgument {
                            key: ParameterKey::Positional,
                            value: Expression::from(vec![Box::new(Term::StringLiteral("Hello World!".to_string()))]),
                            type_declaration: None,
                        }
                    ])),
                ])))]))])),
                return_type: None,
            }
        )));

        let builtins = program::builtins::create_builtins();
        let builtin_variable_scope = builtins.create_scope();

        let program = linker::link_program(parsed, &builtin_variable_scope, &builtins).expect("Linker failed");

        assert_eq!(program.module.function_implementations.len(), 2);
        let ptr = program.module.function_pointers.values().filter(|ptr| &ptr.name == "main").exactly_one().unwrap();
        let implementation = &program.module.function_implementations[&ptr.target];
        assert_eq!(implementation.expression_forest.operations.len(), 2);

        let python_ast = transpiler::python::transpile_program(&program, &builtins).expect("Python transpiler failed");
        let python_string = python_ast.to_string();
        assert!(python_string.contains("def main():"));
        assert!(python_string.contains("print(\"Hello World!\")"));
        assert!(python_string.contains("if __name__ == \"__main__\":"));
        assert!(!python_string.contains("transpile"));

        // TODO Pass a pipe and monitor that "Hello World!" is printed.
        let mut globals = InterpreterGlobals::new(&builtins);
        for module in [&program.module].into_iter().chain(builtins.all_modules()) {
            interpreter::load::module(module, &mut globals);
        }
        interpreter::run::main(&program, &mut globals).expect("Interpreter failed");
    }

    /// This tests generics, algebra and printing.
    #[test]
    fn math() {
        let parsed = parser::parse_program(&"
def main() :: {
    print(1 + 2 'Float32);
};

@transpile
def transpile(transpiler 'Transpiler) :: {
    transpiler.add(main);
};
".to_string());

        let builtins = program::builtins::create_builtins();
        let builtin_variable_scope = builtins.create_scope();

        let program = linker::link_program(parsed, &builtin_variable_scope, &builtins).expect("Linker failed");

        let ptr = program.module.function_pointers.values().filter(|ptr| &ptr.name == "main").exactly_one().unwrap();
        let implementation = &program.module.function_implementations[&ptr.target];

        let python_ast = transpiler::python::transpile_program(&program, &builtins).expect("Python transpiler failed");
        let python_string = python_ast.to_string();
        assert!(python_string.contains("def main():"));
    }
}
