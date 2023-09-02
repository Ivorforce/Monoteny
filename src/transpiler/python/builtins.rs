use uuid::Uuid;
use crate::program::builtins::Builtins;
use crate::program::global::{BuiltinFunctionHint, PrimitiveOperation};
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
    for (fun, hint) in builtins.core.module.builtin_hints.iter() {
        match hint {
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Add, type_ } => {
                namespace.insert_keyword(fun.pointer_id, &String::from("op.add"));
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Subtract, type_ } => {
                namespace.insert_keyword(fun.pointer_id, &String::from("op.sub"));
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Multiply, type_ } => {
                namespace.insert_keyword(fun.pointer_id, &String::from("op.mul"));
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Divide, type_ } => {
                namespace.insert_keyword(fun.pointer_id, &String::from("op.truediv"));
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Log, type_ } => {
                namespace.insert_keyword(fun.pointer_id, &String::from("math.log"));
            }
            _ => {}
        }
    }

    namespace.insert_keyword(builtins.debug.print.pointer_id, &String::from("print"));

    for trait_ in builtins.core.module.traits.keys() {
        // TODO Introduce a package ref system.
        namespace.register_definition(trait_.id, &format!("mn.traits.{}", &trait_.name));

        let trait_namespace = namespace.add_sublevel();
        for fun in trait_.abstract_functions.iter() {
            trait_namespace.register_definition(fun.target.function_id, &fun.name);
        }
    }

    // I don't think we need this. This is just "extend Object: Interface" - this usually is not referred to in code.
    // for module in builtins.all_modules() {
    //     for (trait_, mapping) in module.trait_conformance.declarations.iter() {
    //         for (binding, mapping) in mapping.iter() {
    //             namespace.register_definition(Uuid::new_v4(), &format!("mn.declarations.{}", &trait_.name));
    //         }
    //     }
    // }

    namespace
}