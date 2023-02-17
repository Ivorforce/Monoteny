use uuid::Uuid;
use crate::program::builtins::Builtins;
use crate::program::functions::FunctionCallType;
use crate::transpiler::namespaces;

pub fn create(builtins: &Builtins) -> namespaces::Level {
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
        ("op.add", &builtins.primitives.add),
        ("op.sub", &builtins.primitives.subtract),
        ("op.mul", &builtins.primitives.multiply),
        // TODO This is not true for int types, there it has to be floordiv
        ("op.truediv", &builtins.primitives.divide),
        ("math.log", &builtins.primitives.logarithm),
    ]{
        for fun in functions.values() {
            namespace.insert_keyword(fun.pointer_id, &String::from(name));
        }
    }

    namespace.insert_keyword(builtins.debug.print.pointer_id, &String::from("print"));

    for trait_ in builtins.traits.all.iter() {
        // TODO Introduce a package ref system.
        namespace.register_definition(trait_.id, &format!("mn.traits.{}", &trait_.name));

        let trait_namespace = namespace.add_sublevel();
        for fun in trait_.abstract_functions.iter() {
            trait_namespace.register_definition(fun.function_id, &fun.interface.name);
        }
    }

    for declaration in builtins.global_constants.trait_conformance_declarations.declarations.values().flatten() {
        namespace.register_definition(declaration.id, &format!("mn.declarations.{}", &declaration.trait_.name));
    }

    namespace.register_definition(builtins.math.pi.pointer_id, &String::from("mn.pi"));
    namespace.register_definition(builtins.math.tau.pointer_id, &String::from("mn.tau"));
    namespace.register_definition(builtins.math.e.pointer_id, &String::from("mn.e"));

    namespace
}