use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;
use crate::program::builtins::Builtins;
use crate::program::global::{BuiltinFunctionHint, PrimitiveOperation};
use crate::program::primitives;
use crate::program::types::{TypeProto, TypeUnit};
use crate::transpiler::namespaces;

pub fn create_name_level(builtins: &Builtins, type_ids: &mut HashMap<Box<TypeProto>, Uuid>) -> namespaces::Level {
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
        namespace.insert_keyword(Uuid::new_v4(), keyword);
    }

    for type_name in [
        "bool",
        "np", "op",
    ] {
        namespace.insert_keyword(Uuid::new_v4(), type_name);
    }

    // The operators can normally be referenced as operators (which the transpiler does do).
    // However, if a reference is required, we need to resort to another strategy.
    for (fun, hint) in builtins.core.module.fn_builtin_hints.iter() {
        match hint {
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Add, type_ } => {
                namespace.insert_keyword(fun.function_id, "op.add");
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Subtract, type_ } => {
                namespace.insert_keyword(fun.function_id, "op.sub");
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Multiply, type_ } => {
                namespace.insert_keyword(fun.function_id, "op.mul");
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Divide, type_ } => {
                namespace.insert_keyword(fun.function_id, "op.truediv");
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Log, type_ } => {
                namespace.insert_keyword(fun.function_id, "math.log");
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::ToString, type_ } => {
                namespace.insert_keyword(fun.function_id, "str");
            }
            _ => {}
        }
    }

    namespace.insert_keyword(Uuid::new_v4(), "print");
    
    for trait_ in builtins.core.module.traits.keys() {
        // TODO Introduce a package ref system.
        namespace.register_definition(trait_.id, &format!("mn.traits.{}", &trait_.name));

        let trait_namespace = namespace.add_sublevel();
        for fun in trait_.abstract_functions.values() {
            trait_namespace.register_definition(fun.target.function_id, &fun.name);
        }
    }

    // TODO 128 bit ints are not available in numpy.
    for (struct_, name) in [
        (&builtins.core.traits.String, "str"),
        (&builtins.core.primitives[&primitives::Type::Bool], "bool"),
        (&builtins.core.primitives[&primitives::Type::Int8], "int8"),
        (&builtins.core.primitives[&primitives::Type::Int16], "int16"),
        (&builtins.core.primitives[&primitives::Type::Int32], "int32"),
        (&builtins.core.primitives[&primitives::Type::Int64], "int64"),
        (&builtins.core.primitives[&primitives::Type::UInt8], "uint8"),
        (&builtins.core.primitives[&primitives::Type::UInt16], "uint16"),
        (&builtins.core.primitives[&primitives::Type::UInt32], "uint32"),
        (&builtins.core.primitives[&primitives::Type::UInt64], "uint64"),
        (&builtins.core.primitives[&primitives::Type::Float32], "float32"),
        (&builtins.core.primitives[&primitives::Type::Float64], "float64"),
    ].into_iter() {
        let id = Uuid::new_v4();
        type_ids.insert(TypeProto::unit(TypeUnit::Struct(Rc::clone(struct_))), id);
        namespace.insert_keyword(id, &name.to_string());
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