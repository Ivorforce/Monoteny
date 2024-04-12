#[cfg(test)]
mod tests {
    use std::fs;
    use itertools::Itertools;
    use crate::{interpreter, parser, transpiler};
    use crate::error::RResult;
    use crate::parser::ast::*;

    #[test]
    fn hello_world() -> RResult<()> {
        let file_contents = fs::read_to_string("test-code/hello_world.monoteny").unwrap();
        let (parsed, errors) = parser::parse_program(
            file_contents.as_str()
        )?;
        assert!(errors.is_empty());

        assert_eq!(parsed.statements.len(), 3);

        let Statement::FunctionDeclaration(function) = &parsed.statements[1].as_ref().value.value else {
            panic!();
        };

        assert!(function.interface.return_type.is_none());
        assert_eq!(function.interface.expression.len(), 1);
        assert!(function.interface.expression[0].value == Term::MacroIdentifier("main".to_string()));

        Ok(())
    }

    #[test]
    fn custom_grammar() -> RResult<()> {
        let file_contents = fs::read_to_string("test-code/grammar/custom_grammar.monoteny").unwrap();
        let (parsed, errors) = parser::parse_program(file_contents.as_str())?;
        assert!(errors.is_empty());

        assert_eq!(parsed.statements.len(), 4);

        let Statement::FunctionDeclaration(floor_div) = &parsed.statements[1].as_ref().value.value else {
            panic!();
        };

        match floor_div.interface.expression.iter().map(|t| &t.value).collect_vec()[..] {
            [Term::Identifier(i), Term::Struct(s)] => {
                assert_eq!(i, "_add");
                assert_eq!(s.len(), 2);
            }
            _ => panic!()
        }
        assert_eq!(parsed.statements[1].decorations.len(), 1);

        Ok(())
    }
}
