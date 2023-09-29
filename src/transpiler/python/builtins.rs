use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;
use crate::interpreter::Runtime;
use crate::program::global::{BuiltinFunctionHint, PrimitiveOperation};
use crate::program::primitives;
use crate::program::types::{TypeProto, TypeUnit};
use crate::transpiler::namespaces;
use crate::transpiler::python::representations::{FunctionRepresentation, Representations};


pub fn register(runtime: &Runtime, representations: &mut Representations) -> namespaces::Level {
    let mut namespace = namespaces::Level::new();

    let primitive_map = HashMap::from([
        (primitives::Type::Bool, "Bool"),
        (primitives::Type::Int8, "int8"),
        (primitives::Type::Int16, "int16"),
        (primitives::Type::Int32, "int32"),
        (primitives::Type::Int64, "int64"),
        (primitives::Type::UInt8, "uint8"),
        (primitives::Type::UInt16, "uint16"),
        (primitives::Type::UInt32, "uint32"),
        (primitives::Type::UInt64, "uint64"),
        (primitives::Type::Float32, "float32"),
        (primitives::Type::Float64, "float64"),
    ]);

    // Keywords
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
        namespace.insert_fixed_name(Uuid::new_v4(), keyword);
    }

    // TODO Imports; we should resolve names deeply in the future
    for type_name in [
        "bool",
        "np", "op",
        "math",
    ] {
        namespace.insert_fixed_name(Uuid::new_v4(), type_name);
    }

    // The operators can normally be referenced as operators (which the transpiler does do).
    // However, if a reference is required, we need to resort to another strategy.
    for (head, hint) in runtime.builtins.core.module.fn_builtin_hints.iter() {
        let (higher_order_ref_name, representation) = match hint {
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::EqualTo, type_ } => {
                ("op.eq", FunctionRepresentation::Binary("==".to_string()))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::NotEqualTo, type_ } => {
                ("op.ne", FunctionRepresentation::Binary("!=".to_string()))
            }

            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::GreaterThan, type_ } => {
                ("op.gt", FunctionRepresentation::Binary(">".to_string()))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::LesserThan, type_ } => {
                ("op.lt", FunctionRepresentation::Binary("<".to_string()))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::GreaterThanOrEqual, type_ } => {
                ("op.ge", FunctionRepresentation::Binary(">=".to_string()))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::LesserThanOrEqual, type_ } => {
                ("op.le", FunctionRepresentation::Binary("<=".to_string()))
            }

            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::And, type_ } => {
                ("op.and_", FunctionRepresentation::Binary("and".to_string()))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Or, type_ } => {
                ("op.or_", FunctionRepresentation::Binary("or".to_string()))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Not, type_ } => {
                ("op.not_", FunctionRepresentation::Unary("not".to_string()))
            }

            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Negative, type_ } => {
                ("op.neg", FunctionRepresentation::Unary("-".to_string()))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Add, type_ } => {
                ("op.add", FunctionRepresentation::Binary("+".to_string()))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Subtract, type_ } => {
                ("op.sub", FunctionRepresentation::Binary("-".to_string()))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Multiply, type_ } => {
                ("op.mul", FunctionRepresentation::Binary("*".to_string()))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Divide, type_ } => {
                match type_.is_int() {
                    true => ("op.truediv", FunctionRepresentation::Binary("//".to_string())),
                    false => ("op.div", FunctionRepresentation::Binary("/".to_string())),
                }
            }

            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Modulo, type_ } => {
                ("op.mod", FunctionRepresentation::Binary("%".to_string()))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Exp, type_ } => {
                ("op.pow", FunctionRepresentation::Binary("**".to_string()))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Log, type_ } => {
                ("math.log", FunctionRepresentation::FunctionCall("math.log".to_string()))
            }

            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::ToString, type_ } => {
                ("str", FunctionRepresentation::FunctionCall("str".to_string()))
            }

            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::ParseIntString, type_ } => {
                if let Some(builtin_name) = primitive_map.get(type_) {
                    (builtin_name.clone(), FunctionRepresentation::FunctionCall(builtin_name.to_string()))
                }
                else {
                    continue
                }
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::ParseFloatString, type_ } => {
                if let Some(builtin_name) = primitive_map.get(type_) {
                    (builtin_name.clone(), FunctionRepresentation::FunctionCall(builtin_name.to_string()))
                }
                else {
                    continue
                }
            }

            BuiltinFunctionHint::True => {
                ("True", FunctionRepresentation::Constant("True".to_string()))
            }
            BuiltinFunctionHint::False => {
                ("False", FunctionRepresentation::Constant("False".to_string()))
            }

            BuiltinFunctionHint::Constructor => {
                continue
            }
        };

        representations.builtin_functions.insert(Rc::clone(head));
        representations.function_representations.insert(Rc::clone(head), representation);
        namespace.insert_fixed_name(head.function_id, higher_order_ref_name);
    }

    for (struct_, name) in [
        (&runtime.builtins.core.traits.String, "str"),
    ].into_iter() {
        let id = Uuid::new_v4();
        representations.type_ids.insert(TypeProto::unit(TypeUnit::Struct(Rc::clone(struct_))), id);
        namespace.insert_fixed_name(id, &name.to_string());
    }

    for (primitive, name) in primitive_map.iter() {
        let id = Uuid::new_v4();
        let struct_ = &runtime.builtins.core.primitives[primitive];
        representations.type_ids.insert(TypeProto::unit(TypeUnit::Struct(Rc::clone(struct_))), id);
        namespace.insert_fixed_name(id, &name.to_string());
    }

    for ptr in runtime.source.module_by_name["math"].fn_pointers.values() {
        let representation = match ptr.name.as_str() {
            "factorial" => FunctionRepresentation::FunctionCall("math.factorial".to_string()),

            "sin" => FunctionRepresentation::FunctionCall("math.sin".to_string()),
            "cos" => FunctionRepresentation::FunctionCall("math.cos".to_string()),
            "tan" => FunctionRepresentation::FunctionCall("math.tan".to_string()),
            "sinh" => FunctionRepresentation::FunctionCall("math.sinh".to_string()),
            "cosh" => FunctionRepresentation::FunctionCall("math.cosh".to_string()),
            "tanh" => FunctionRepresentation::FunctionCall("math.tanh".to_string()),
            "arcsin" => FunctionRepresentation::FunctionCall("math.asin".to_string()),
            "arccos" => FunctionRepresentation::FunctionCall("math.acos".to_string()),
            "arctan" => FunctionRepresentation::FunctionCall("math.atan".to_string()),
            "arcsinh" => FunctionRepresentation::FunctionCall("math.asinh".to_string()),
            "arccosh" => FunctionRepresentation::FunctionCall("math.acosh".to_string()),
            "arctanh" => FunctionRepresentation::FunctionCall("math.atanh".to_string()),

            "ceil" => FunctionRepresentation::FunctionCall("math.ceil".to_string()),
            "floor" => FunctionRepresentation::FunctionCall("math.floor".to_string()),
            "round" => FunctionRepresentation::FunctionCall("round".to_string()),

            "abs" => FunctionRepresentation::FunctionCall("abs".to_string()),
            _ => continue,
        };

        representations.builtin_functions.insert(Rc::clone(&ptr.target));
        namespace.insert_fixed_name(ptr.target.function_id, representation.name());
        representations.function_representations.insert(Rc::clone(&ptr.target), representation);
    }

    for ptr in runtime.source.module_by_name["debug"].fn_pointers.values() {
        let representation = match ptr.name.as_str() {
            "_write_line" => FunctionRepresentation::FunctionCall("print".to_string()),
            "panic" => FunctionRepresentation::FunctionCall("exit".to_string()),
            "todo" => FunctionRepresentation::FunctionCall("exit".to_string()),
            "unreachable" => FunctionRepresentation::FunctionCall("exit".to_string()),
            _ => continue,
        };

        representations.builtin_functions.insert(Rc::clone(&ptr.target));
        namespace.insert_fixed_name(ptr.target.function_id, representation.name());
        representations.function_representations.insert(Rc::clone(&ptr.target), representation);
    }

    for ptr in runtime.source.module_by_name["strings"].fn_pointers.values() {
        let representation = match ptr.name.as_str() {
            "add" => FunctionRepresentation::FunctionCall("op.add".to_string()),
            _ => continue,
        };

        representations.builtin_functions.insert(Rc::clone(&ptr.target));
        namespace.insert_fixed_name(ptr.target.function_id, representation.name());
        representations.function_representations.insert(Rc::clone(&ptr.target), representation);
    }

    namespace
}