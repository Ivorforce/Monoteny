#[cfg(test)]
mod tests {
    use crate::{linker, parser, program};
    use crate::parser::abstract_syntax::*;
    use crate::program::functions::ParameterKey;

    #[test]
    fn hello_world() {
        let program = parser::parse_program(&"
@main
def main() :: {
    print(\"Hello World!\");
}
".to_string());
        assert_eq!(program, Program {
            global_statements: vec![
                Box::new(GlobalStatement::FunctionDeclaration(Box::new(
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
                )))
            ],
        });

        let builtins = program::builtins::create_builtins();
        let builtin_variable_scope = builtins.create_scope();

        let linked = linker::link_program(program, &builtin_variable_scope, &builtins).expect("Linker failed");

        assert_eq!(linked.function_implementations.len(), 1);
        let implementation = linked.function_implementations.values().next().unwrap();
        assert_eq!(implementation.pointer.name, "main");
        assert_eq!(implementation.expression_forest.operations.len(), 2);
    }
}
