#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use itertools::Itertools;
    use crate::{interpreter, parser, transpiler};
    use crate::error::RResult;
    use crate::interpreter::Runtime;
    use crate::parser::ast::*;
    use crate::program::module::module_name;
    use crate::transpiler::LanguageContext;

    #[test]
    fn hello_world() -> RResult<()> {
        let (parsed, errors) = parser::parse_program("
    use!(module!(\"common\"));

    def main! :: {
        write_line(\"Hello World!\");
    };

    def transpile! :: {
        transpiler.add(main);
    };
    ")?;
        assert!(errors.is_empty());

        assert_eq!(parsed.global_statements.len(), 3);

        let Statement::FunctionDeclaration(function) = &parsed.global_statements[1].as_ref().value.value else {
            panic!();
        };

        assert!(function.interface.return_type.is_none());
        assert_eq!(function.interface.expression.len(), 1);
        assert!(function.interface.expression[0].value == Term::MacroIdentifier("main".to_string()));

        Ok(())
    }

    #[test]
    fn custom_grammar() -> RResult<()> {
        let (parsed, errors) = parser::parse_program("
    precedence_order!([
        MultiplicationPrecedence(Left),
    ]);

    ![pattern(lhs /_ rhs, MultiplicationPrecedence)]
    def floor_div(lhs '$Real, rhs '$Real) -> $Real :: floor(divide(lhs, rhs));

    def main! :: {
        let a 'Float32 = 1 /_ 2;
    };
    ")?;
        assert!(errors.is_empty());

        assert_eq!(parsed.global_statements.len(), 3);

        let Statement::FunctionDeclaration(floor_div) = &parsed.global_statements[1].as_ref().value.value else {
            panic!();
        };

        match floor_div.interface.expression.iter().map(|t| &t.value).collect_vec()[..] {
            [Term::Identifier(i), Term::Struct(s)] => {
                assert_eq!(i, "floor_div");
                assert_eq!(s.len(), 2);
            }
            _ => panic!()
        }
        assert_eq!(parsed.global_statements[1].decorations.len(), 1);

        Ok(())
    }
}
