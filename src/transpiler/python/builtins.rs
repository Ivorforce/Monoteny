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
        namespace.insert_keyword(Uuid::new_v4(), &String::from(keyword));
    }

    for type_name in [
        // Pretend these are built-in; actually we're borrowing from numpy with an import.
        "int8", "int16", "int32", "int64", "int128",
        "uint8", "uint16", "uint32", "uint64", "uint128",
        "float32", "float64",
        // This is actually built-in but we don't want to accidentally shadow it.
        "bool",
        "np", "op",
    ] {
        namespace.insert_keyword(Uuid::new_v4(), &String::from(type_name));
    }

    // The operators can normally be referenced as operators (which the transpiler does do).
    // However, if a reference is required, we need to resort to another strategy.
    for (name, functions) in [
        ("op.add", &builtins.operators.add),
        ("op.sub", &builtins.operators.subtract),
        ("op.mul", &builtins.operators.multiply),
        // TODO This is not true for int types, there it has to be floordiv
        ("op.truediv", &builtins.operators.divide),
    ]{
        for fun in functions {
            namespace.insert_keyword(fun.pointer_id, &String::from(name));
        }
    }

    namespace.insert_keyword(builtins.functions.print.pointer_id, &String::from("print"));

    namespace
}