use std::collections::HashMap;
use std::rc::Rc;
use strum::IntoEnumIterator;
use uuid::Uuid;
use crate::interpreter::Runtime;
use crate::program::global::{BuiltinFunctionHint, PrimitiveOperation};
use crate::program::module::module_name;
use crate::program::primitives;
use crate::program::types::{TypeProto, TypeUnit};
use crate::transpiler::python::{Context, keywords};
use crate::transpiler::python::keywords::{KEYWORD_IDS, PSEUDO_KEYWORD_IDS};
use crate::transpiler::python::representations::FunctionForm;


pub fn register_global(runtime: &Runtime, context: &mut Context) {
    let representations = &mut context.representations;
    let global = &mut context.builtin_global_namespace;
    let member = &mut context.builtin_member_namespace;

    keywords::register(global);
    keywords::register(member);

    let primitive_map = HashMap::from([
        (primitives::Type::Bool, "bool"),
        (primitives::Type::Int(8), "int8"),
        (primitives::Type::Int(16), "int16"),
        (primitives::Type::Int(32), "int32"),
        (primitives::Type::Int(64), "int64"),
        (primitives::Type::UInt(8), "uint8"),
        (primitives::Type::UInt(16), "uint16"),
        (primitives::Type::UInt(32), "uint32"),
        (primitives::Type::UInt(64), "uint64"),
        (primitives::Type::Float(32), "float32"),
        (primitives::Type::Float(64), "float64"),
    ]);

    // The operators can normally be referenced as operators (which the transpiler does do).
    // However, if a reference is required, we need to resort to another strategy.
    for (head, hint) in runtime.source.module_by_name[&module_name("builtins")].fn_builtin_hints.iter() {
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
                ("math.log", FunctionForm::FunctionCall(PSEUDO_KEYWORD_IDS["math.log"]))
            }

            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::ToString, type_ } => {
                ("str", FunctionForm::FunctionCall(PSEUDO_KEYWORD_IDS["str"]))
            }

            BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::ParseIntString, type_ }
            | BuiltinFunctionHint::PrimitiveOperation { operation: PrimitiveOperation::ParseRealString, type_ } => {
                if let Some(builtin_name) = primitive_map.get(type_) {
                    (builtin_name.clone(), FunctionForm::FunctionCall(PSEUDO_KEYWORD_IDS[builtin_name]))
                }
                else {
                    continue
                }
            }

            BuiltinFunctionHint::Constructor(_) => continue,
            BuiltinFunctionHint::GetMemberField(_) => continue,
            BuiltinFunctionHint::SetMemberField(_) => continue,
        };

        representations.builtin_functions.insert(Rc::clone(head));
        representations.function_representations.insert(Rc::clone(head), representation);
    }

    for (struct_, id) in [
        (&runtime.builtins.traits.String, PSEUDO_KEYWORD_IDS["str"]),
    ].into_iter() {
        representations.type_ids.insert(TypeProto::unit_struct(struct_), id);
    }

    for (primitive, name) in primitive_map.iter() {
        let struct_ = &runtime.builtins.primitives[primitive];
        representations.type_ids.insert(TypeProto::unit_struct(struct_), PSEUDO_KEYWORD_IDS[name]);
    }

    // TODO Some of these sneakily convert the type - especially float to int and vice versa.
    for (function, representation) in runtime.source.module_by_name[&module_name("common.math")].fn_representations.iter() {
        let id = match representation.name.as_str() {
            "factorial" => PSEUDO_KEYWORD_IDS["math.factorial"],
            "sin" => PSEUDO_KEYWORD_IDS["math.sin"],
            "cos" => PSEUDO_KEYWORD_IDS["math.cos"],
            "tan" => PSEUDO_KEYWORD_IDS["math.tan"],
            "sinh" => PSEUDO_KEYWORD_IDS["math.sinh"],
            "cosh" => PSEUDO_KEYWORD_IDS["math.cosh"],
            "tanh" => PSEUDO_KEYWORD_IDS["math.tanh"],
            "arcsin" => PSEUDO_KEYWORD_IDS["math.asin"],
            "arccos" => PSEUDO_KEYWORD_IDS["math.acos"],
            "arctan" => PSEUDO_KEYWORD_IDS["math.atan"],
            "arcsinh" => PSEUDO_KEYWORD_IDS["math.asinh"],
            "arccosh" => PSEUDO_KEYWORD_IDS["math.acosh"],
            "arctanh" => PSEUDO_KEYWORD_IDS["math.atanh"],

            "ceil" => PSEUDO_KEYWORD_IDS["math.ceil"],
            "floor" => PSEUDO_KEYWORD_IDS["math.floor"],
            "round" => PSEUDO_KEYWORD_IDS["round"],

            "abs" => PSEUDO_KEYWORD_IDS["abs"],
            _ => continue,
        };

        representations.builtin_functions.insert(Rc::clone(function));
        // By the time we need other representations hopefully we can use object namespaces
        representations.function_representations.insert(Rc::clone(function), FunctionForm::FunctionCall(id));
    }

    for (function, representation) in runtime.source.module_by_name[&module_name("core.debug")].fn_representations.iter() {
        let id = match representation.name.as_str() {
            "_write_line" => PSEUDO_KEYWORD_IDS["print"],
            "_exit_with_error" => PSEUDO_KEYWORD_IDS["exit"],
            _ => continue,
        };

        representations.builtin_functions.insert(Rc::clone(function));
        representations.function_representations.insert(Rc::clone(function), FunctionForm::FunctionCall(id));
    }

    for (function, representation) in runtime.source.module_by_name[&module_name("core.strings")].fn_representations.iter() {
        let (higher_order_name, id) = match representation.name.as_str() {
            "add" => ("op.add", FunctionForm::Binary(KEYWORD_IDS["+"])),
            _ => continue,
        };

        representations.builtin_functions.insert(Rc::clone(function));
        representations.function_representations.insert(Rc::clone(function), id);
    }

    for (function, representation) in runtime.source.module_by_name[&module_name("core.bool")].fn_representations.iter() {
        let (higher_order_name, id) = match representation.name.as_str() {
            "true" => ("True", FunctionForm::Constant(KEYWORD_IDS["True"])),
            "false" => ("False", FunctionForm::Constant(KEYWORD_IDS["False"])),
            _ => continue,
        };

        representations.builtin_functions.insert(Rc::clone(function));
        representations.function_representations.insert(Rc::clone(function), id);
    }
}
