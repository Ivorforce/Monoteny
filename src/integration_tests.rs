#[cfg(test)]
mod tests {
    use std::io::BufWriter;
    use itertools::Itertools;
    use crate::{interpreter, linker, parser, program, transpiler};
    use crate::parser::abstract_syntax::*;
    use crate::program::functions::ParameterKey;

    /// This tests the transpiler, interpreter and function calls.
    #[test]
    fn hello_world() {
        let parsed = parser::parse_program(&"
@main
def main() :: {
    print(\"Hello World!\");
}

@transpile
def transpile(transpiler 'Transpiler) :: {
    transpiler.add(main);
}
".to_string());
        assert_eq!(parsed.global_statements.len(), 2);
        assert_eq!(parsed.global_statements[0].as_ref(), &GlobalStatement::FunctionDeclaration(Box::new(
            Function {
                target_type: None,
                identifier: "main".to_string(),
                parameters: vec![],
                decorators: vec!["main".to_string()],
                body: vec![
                    Box::new(Statement::Expression(vec![
                        Box::new(Term::Identifier("print".to_string())),
                        Box::new(Term::Struct(vec![
                            StructArgument {
                                key: ParameterKey::Positional,
                                value: vec![Box::new(Term::StringLiteral("Hello World!".to_string()))],
                                type_declaration: None,
                            }
                        ]))
                    ]))
                ],
                return_type: None,
            }
        )));

        let builtins = program::builtins::create_builtins();
        let builtin_variable_scope = builtins.create_scope();

        let linked = linker::link_program(parsed, &builtin_variable_scope, &builtins).expect("Linker failed");

        assert_eq!(linked.function_implementations.len(), 2);
        let implementation = linked.function_implementations.values().filter(|x| &x.pointer.name == "main").next().unwrap();
        assert_eq!(implementation.pointer.name, "main");
        assert_eq!(implementation.expression_forest.operations.len(), 2);

        let mut buf = BufWriter::new(Vec::new());
        transpiler::python::transpile_program(&mut buf, &linked, &builtins).expect("Python transpiler failed");
        let python_program = String::from_utf8(buf.into_inner().unwrap()).unwrap();
        assert!(python_program.contains("def main():"));
        assert!(python_program.contains("print(\"Hello World!\")"));
        assert!(python_program.contains("if __name__ == \"__main__\":"));
        assert!(!python_program.contains("transpile"));

        // TODO Pass a pipe and monitor that "Hello World!" is printed.
        interpreter::run::main(&linked, &builtins);
    }

    /// This tests generics, algebra and printing.
    #[test]
    fn math() {
        let parsed = parser::parse_program(&"
def main() :: {
    print(1 + 2 'Float32);
}

@transpile
def transpile(transpiler 'Transpiler) :: {
    transpiler.add(main);
}
".to_string());

        let builtins = program::builtins::create_builtins();
        let builtin_variable_scope = builtins.create_scope();

        let linked = linker::link_program(parsed, &builtin_variable_scope, &builtins).expect("Linker failed");
        let implementation = linked.function_implementations.values().filter(|x| &x.pointer.name == "main").next().unwrap();

        let mut buf = BufWriter::new(Vec::new());
        transpiler::python::transpile_program(&mut buf, &linked, &builtins).expect("Python transpiler failed");
        let python_program = String::from_utf8(buf.into_inner().unwrap()).unwrap();
        assert!(python_program.contains("def main():"));
    }
}
