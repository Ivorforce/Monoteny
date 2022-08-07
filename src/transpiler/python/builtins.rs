use uuid::Uuid;
use crate::program::builtins::TenLangBuiltins;
use crate::transpiler::namespaces;

pub fn create(builtins: &TenLangBuiltins) -> namespaces::Level {
    let mut namespace = namespaces::Level::new();

    for keyword in [
        "False", "class", "from", "or",
        "None", "continue", "global", "pass",
        "True", "def", "if", "raise", "and",
        "del", "import", "return", "as",
        "elif", "in", "try", "assert",
        "else", "is", "while", "async",
        "except", "lambda", "with", "await",
        "finally", "nonlocal", "yield", "break",
        "for", "not"
    ] {
        // Don't really need an ID but it's easy to just do it like this here.
        namespace.register_definition(Uuid::new_v4(), &String::from(keyword));
    }

    namespace.register_definition(builtins.functions.print.id, &String::from("print"));

    namespace
}