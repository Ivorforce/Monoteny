use std::collections::HashMap;
use std::rc::Rc;
use strum::IntoEnumIterator;
use uuid::Uuid;
use crate::interpreter::Runtime;
use crate::program::global::{BuiltinFunctionHint, PrimitiveOperation};
use crate::program::primitives;
use crate::program::types::{TypeProto, TypeUnit};
use crate::transpiler::python::{Context, keywords};
use crate::transpiler::python::keywords::KEYWORD_IDS;
use crate::transpiler::python::representations::FunctionForm;


pub fn register_global(runtime: &Runtime, context: &mut Context) {
    let representations = &mut context.representations;
    let global = &mut context.builtin_global_namespace;
    let member = &mut context.builtin_member_namespace;

    keywords::register(global);
    keywords::register(member);

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

    // TODO Imports; we should resolve names deeply in the future
    for type_name in [
        "bool",
        "np", "op",
        "math",
    ] {
        global.insert_fixed_name(Uuid::new_v4(), type_name);
    }

    // The operators can normally be referenced as operators (which the transpiler does do).
    // However, if a reference is required, we need to resort to another strategy.
    for (head, hint) in runtime.builtins.module.fn_builtin_hints.iter() {
        let (higher_order_ref_name, representation) = match hint {
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::EqualTo, type_ } => {
                ("op.eq", FunctionForm::Binary(KEYWORD_IDS["=="]))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::NotEqualTo, type_ } => {
                ("op.ne", FunctionForm::Binary(KEYWORD_IDS["!="]))
            }

            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::GreaterThan, type_ } => {
                ("op.gt", FunctionForm::Binary(KEYWORD_IDS[">"]))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::LesserThan, type_ } => {
                ("op.lt", FunctionForm::Binary(KEYWORD_IDS["<"]))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::GreaterThanOrEqual, type_ } => {
                ("op.ge", FunctionForm::Binary(KEYWORD_IDS[">="]))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::LesserThanOrEqual, type_ } => {
                ("op.le", FunctionForm::Binary(KEYWORD_IDS["<="]))
            }

            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::And, type_ } => {
                ("op.and_", FunctionForm::Binary(KEYWORD_IDS["and"]))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Or, type_ } => {
                ("op.or_", FunctionForm::Binary(KEYWORD_IDS["or"]))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Not, type_ } => {
                ("op.not_", FunctionForm::Unary(KEYWORD_IDS["not"]))
            }

            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Negative, type_ } => {
                ("op.neg", FunctionForm::Unary(KEYWORD_IDS["-"]))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Add, type_ } => {
                ("op.add", FunctionForm::Binary(KEYWORD_IDS["+"]))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Subtract, type_ } => {
                ("op.sub", FunctionForm::Binary(KEYWORD_IDS["-"]))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Multiply, type_ } => {
                ("op.mul", FunctionForm::Binary(KEYWORD_IDS["*"]))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Divide, type_ } => {
                match type_.is_int() {
                    true => ("op.truediv", FunctionForm::Binary(KEYWORD_IDS["//"])),
                    false => ("op.div", FunctionForm::Binary(KEYWORD_IDS["//"])),
                }
            }

            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Modulo, type_ } => {
                ("op.mod", FunctionForm::Binary(KEYWORD_IDS["%"]))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Exp, type_ } => {
                ("op.pow", FunctionForm::Binary(KEYWORD_IDS["**"]))
            }
            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::Log, type_ } => {
                global.insert_fixed_name(head.function_id, "math.log");
                ("math.log", FunctionForm::FunctionCall(head.function_id))
            }

            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::ToString, type_ } => {
                global.insert_fixed_name(head.function_id, "str");
                ("str", FunctionForm::FunctionCall(head.function_id))
            }

            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::ParseIntString, type_ }
            | BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::ParseRealString, type_ } => {
                if let Some(builtin_name) = primitive_map.get(type_) {
                    global.insert_fixed_name(head.function_id, builtin_name);
                    (builtin_name.clone(), FunctionForm::FunctionCall(head.function_id))
                }
                else {
                    continue
                }
            }

            BuiltinFunctionHint::True => {
                ("True", FunctionForm::Constant(KEYWORD_IDS["True"]))
            }
            BuiltinFunctionHint::False => {
                ("False", FunctionForm::Constant(KEYWORD_IDS["False"]))
            }

            BuiltinFunctionHint::Constructor(_) => continue,
            BuiltinFunctionHint::Getter(_) => continue,
            BuiltinFunctionHint::Setter(_) => continue,
        };

        representations.builtin_functions.insert(Rc::clone(head));
        representations.function_representations.insert(Rc::clone(head), representation);
        global.insert_fixed_name(head.function_id, higher_order_ref_name);
    }

    for (struct_, name) in [
        (&runtime.builtins.traits.String, "str"),
    ].into_iter() {
        let id = Uuid::new_v4();
        representations.type_ids.insert(TypeProto::unit(TypeUnit::Struct(Rc::clone(struct_))), id);
        global.insert_fixed_name(id, &name.to_string());
    }

    for (primitive, name) in primitive_map.iter() {
        let id = Uuid::new_v4();
        let struct_ = &runtime.builtins.primitives[primitive];
        representations.type_ids.insert(TypeProto::unit(TypeUnit::Struct(Rc::clone(struct_))), id);
        global.insert_fixed_name(id, &name.to_string());
    }

    // TODO Some of these sneakily convert the type - especially float to int and vice versa.
    for ptr in runtime.source.module_by_name["math"].fn_pointers.values() {
        let representation = match ptr.name.as_str() {
            "factorial" => "math.factorial",
            "sin" => "math.sin",
            "cos" => "math.cos",
            "tan" => "math.tan",
            "sinh" => "math.sinh",
            "cosh" => "math.cosh",
            "tanh" => "math.tanh",
            "arcsin" => "math.asin",
            "arccos" => "math.acos",
            "arctan" => "math.atan",
            "arcsinh" => "math.asinh",
            "arccosh" => "math.acosh",
            "arctanh" => "math.atanh",

            "ceil" => "math.ceil",
            "floor" => "math.floor",
            "round" => "round",

            "abs" => "abs",
            _ => continue,
        };

        representations.builtin_functions.insert(Rc::clone(&ptr.target));
        global.insert_fixed_name(ptr.target.function_id, representation);
        // By the time we need other representations hopefully we can use object namespaces
        representations.function_representations.insert(Rc::clone(&ptr.target), FunctionForm::FunctionCall(ptr.target.function_id));
    }

    for ptr in runtime.source.module_by_name["debug"].fn_pointers.values() {
        let representation = match ptr.name.as_str() {
            "_write_line" => "print",
            "_exit_with_error" => "exit",
            _ => continue,
        };

        representations.builtin_functions.insert(Rc::clone(&ptr.target));
        global.insert_fixed_name(ptr.target.function_id, representation);
        representations.function_representations.insert(Rc::clone(&ptr.target), FunctionForm::FunctionCall(ptr.target.function_id));
    }

    for ptr in runtime.source.module_by_name["strings"].fn_pointers.values() {
        let (higher_order_name, representation) = match ptr.name.as_str() {
            "add" => ("op.add", FunctionForm::Binary(KEYWORD_IDS["+"])),
            _ => continue,
        };

        representations.builtin_functions.insert(Rc::clone(&ptr.target));
        global.insert_fixed_name(ptr.target.function_id, higher_order_name);
        representations.function_representations.insert(Rc::clone(&ptr.target), representation);
    }
}
