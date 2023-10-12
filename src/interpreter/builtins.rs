use std::alloc::{alloc, Layout};
use std::path::PathBuf;
use std::rc::Rc;
use monoteny_macro::{bin_op, parse_op, un_op, fun_op, to_string_op, load_constant};
use std::str::FromStr;
use guard::guard;
use uuid::Uuid;
use crate::error::RResult;
use crate::interpreter::{FunctionInterpreterImpl, Runtime, Value};
use crate::program::functions::FunctionHead;
use crate::program::global::{BuiltinFunctionHint, PrimitiveOperation};
use crate::program::module::module_name;
use crate::program::primitives;
use crate::program::types::TypeUnit;

pub fn load(runtime: &mut Runtime) -> RResult<()> {
    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Monoteny files --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    runtime.repository.add("core", PathBuf::from("monoteny"));
    runtime.get_or_load_module(&module_name("core"))?;

    for (function, representation) in runtime.source.module_by_name[&module_name("core.debug")].fn_representations.iter() {
        runtime.function_evaluators.insert(function.unwrap_id(), match representation.name.as_str() {
            "_write_line" => Rc::new(move |interpreter, expression_id, binding| {{
                unsafe {{
                    let args = interpreter.implementation.expression_forest.arguments[&expression_id].clone();
                    let arg = interpreter.evaluate(args[0]).unwrap();
                    println!("{}", *(arg.data as *const String));

                    None
                }}
            }}),
            "_exit_with_error" => Rc::new(move |interpreter, expression_id, binding| {{
                unsafe {{
                    let args = interpreter.implementation.expression_forest.arguments[&expression_id].clone();
                    let arg = interpreter.evaluate(args[0]).unwrap();

                    panic!("{}", *(arg.data as *const String));
                }}
            }}),
            _ => continue,
        });
    }

    for (function, representation) in runtime.source.module_by_name[&module_name("core.transpilation")].fn_representations.iter() {
        runtime.function_evaluators.insert(
            function.function_id,
            Rc::new(move |interpreter, expression_id, binding| {
                unsafe {
                    let arguments = interpreter.evaluate_arguments(expression_id);

                    // This may cause a SIGSEV if the callback pointer is invalidated. This should not happen as long as
                    //  nobody owns a Transpiler object outside of its lifetime.
                    let transpiler_callback = *(arguments[0].data as *const &dyn Fn(Rc<FunctionHead>, &Runtime));

                    let arg = &arguments[1];
                    let arg_id = &interpreter.implementation.expression_forest.arguments[&expression_id][1];
                    let arg_type = interpreter.implementation.type_forest.get_unit(arg_id).unwrap();

                    // TODO Once we have a Function supertype we can remove this check.
                    match arg_type {
                        TypeUnit::Function(f) => {},
                        _ => panic!("Argument to transpiler.add is not a function: {:?}", arg_type)
                    };

                    let implementation_id = *(arg.data as *const Uuid);
                    guard!(let implementation = &interpreter.runtime.source.fn_heads[&implementation_id] else {
                    panic!("Couldn't find function head: {}", implementation_id)
                });
                    transpiler_callback(Rc::clone(implementation), &interpreter.runtime);

                    return None;
                }
            })
        );
    }

    for (function, representation) in runtime.source.module_by_name[&module_name("core.bool")].fn_representations.iter() {
        runtime.function_evaluators.insert(function.unwrap_id(), match representation.name.as_str() {
            "true" => load_constant!(bool true),
            "false" => load_constant!(bool false),
            _ => continue,
        });
    }

    let builtins = &runtime.builtins;

    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Math --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    for (head, builtin_hint) in builtins.module.fn_builtin_hints.iter() {
        runtime.function_evaluators.insert(head.unwrap_id(), match builtin_hint {
            BuiltinFunctionHint::PrimitiveOperation { type_, operation } => {
                create_primitive_op(type_.clone(), operation.clone())
            }
            BuiltinFunctionHint::Constructor(_) => todo!(),
            BuiltinFunctionHint::Getter(_) => todo!(),
            BuiltinFunctionHint::Setter(_) => todo!(),
        });
    }

    Ok(())
}

pub fn create_primitive_op(type_: primitives::Type, operation: PrimitiveOperation) -> FunctionInterpreterImpl {
    match (type_, operation) {
        // -------------------------------------- Bool --------------------------------------
        (primitives::Type::Bool, PrimitiveOperation::And) => bin_op!(bool && bool),
        (primitives::Type::Bool, PrimitiveOperation::Or) => bin_op!(bool || bool),
        (primitives::Type::Bool, PrimitiveOperation::Not) => un_op!(bool !),

        // -------------------------------------- Comparison --------------------------------------
        (primitives::Type::Bool, PrimitiveOperation::EqualTo) => bin_op!(bool == bool),
        (primitives::Type::UInt8, PrimitiveOperation::EqualTo) => bin_op!(u8 == bool),
        (primitives::Type::UInt16, PrimitiveOperation::EqualTo) => bin_op!(u16 == bool),
        (primitives::Type::UInt32, PrimitiveOperation::EqualTo) => bin_op!(u32 == bool),
        (primitives::Type::UInt64, PrimitiveOperation::EqualTo) => bin_op!(u64 == bool),
        (primitives::Type::UInt128, PrimitiveOperation::EqualTo) => bin_op!(u128 == bool),
        (primitives::Type::Int8, PrimitiveOperation::EqualTo) => bin_op!(i8 == bool),
        (primitives::Type::Int16, PrimitiveOperation::EqualTo) => bin_op!(i16 == bool),
        (primitives::Type::Int32, PrimitiveOperation::EqualTo) => bin_op!(i32 == bool),
        (primitives::Type::Int64, PrimitiveOperation::EqualTo) => bin_op!(i64 == bool),
        (primitives::Type::Int128, PrimitiveOperation::EqualTo) => bin_op!(i128 == bool),
        (primitives::Type::Float32, PrimitiveOperation::EqualTo) => bin_op!(f32 == bool),
        (primitives::Type::Float64, PrimitiveOperation::EqualTo) => bin_op!(f64 == bool),

        (primitives::Type::Bool, PrimitiveOperation::NotEqualTo) => bin_op!(bool != bool),
        (primitives::Type::UInt8, PrimitiveOperation::NotEqualTo) => bin_op!(u8 != bool),
        (primitives::Type::UInt16, PrimitiveOperation::NotEqualTo) => bin_op!(u16 != bool),
        (primitives::Type::UInt32, PrimitiveOperation::NotEqualTo) => bin_op!(u32 != bool),
        (primitives::Type::UInt64, PrimitiveOperation::NotEqualTo) => bin_op!(u64 != bool),
        (primitives::Type::UInt128, PrimitiveOperation::NotEqualTo) => bin_op!(u128 != bool),
        (primitives::Type::Int8, PrimitiveOperation::NotEqualTo) => bin_op!(i8 != bool),
        (primitives::Type::Int16, PrimitiveOperation::NotEqualTo) => bin_op!(i16 != bool),
        (primitives::Type::Int32, PrimitiveOperation::NotEqualTo) => bin_op!(i32 != bool),
        (primitives::Type::Int64, PrimitiveOperation::NotEqualTo) => bin_op!(i64 != bool),
        (primitives::Type::Int128, PrimitiveOperation::NotEqualTo) => bin_op!(i128 != bool),
        (primitives::Type::Float32, PrimitiveOperation::NotEqualTo) => bin_op!(f32 != bool),
        (primitives::Type::Float64, PrimitiveOperation::NotEqualTo) => bin_op!(f64 != bool),

        (primitives::Type::UInt8, PrimitiveOperation::GreaterThan) => bin_op!(u8 > bool),
        (primitives::Type::UInt16, PrimitiveOperation::GreaterThan) => bin_op!(u16 > bool),
        (primitives::Type::UInt32, PrimitiveOperation::GreaterThan) => bin_op!(u32 > bool),
        (primitives::Type::UInt64, PrimitiveOperation::GreaterThan) => bin_op!(u64 > bool),
        (primitives::Type::UInt128, PrimitiveOperation::GreaterThan) => bin_op!(u128 > bool),
        (primitives::Type::Int8, PrimitiveOperation::GreaterThan) => bin_op!(i8 > bool),
        (primitives::Type::Int16, PrimitiveOperation::GreaterThan) => bin_op!(i16 > bool),
        (primitives::Type::Int32, PrimitiveOperation::GreaterThan) => bin_op!(i32 > bool),
        (primitives::Type::Int64, PrimitiveOperation::GreaterThan) => bin_op!(i64 > bool),
        (primitives::Type::Int128, PrimitiveOperation::GreaterThan) => bin_op!(i128 > bool),
        (primitives::Type::Float32, PrimitiveOperation::GreaterThan) => bin_op!(f32 > bool),
        (primitives::Type::Float64, PrimitiveOperation::GreaterThan) => bin_op!(f64 > bool),

        (primitives::Type::UInt8, PrimitiveOperation::GreaterThanOrEqual) => bin_op!(u8 >= bool),
        (primitives::Type::UInt16, PrimitiveOperation::GreaterThanOrEqual) => bin_op!(u16 >= bool),
        (primitives::Type::UInt32, PrimitiveOperation::GreaterThanOrEqual) => bin_op!(u32 >= bool),
        (primitives::Type::UInt64, PrimitiveOperation::GreaterThanOrEqual) => bin_op!(u64 >= bool),
        (primitives::Type::UInt128, PrimitiveOperation::GreaterThanOrEqual) => bin_op!(u128 >= bool),
        (primitives::Type::Int8, PrimitiveOperation::GreaterThanOrEqual) => bin_op!(i8 >= bool),
        (primitives::Type::Int16, PrimitiveOperation::GreaterThanOrEqual) => bin_op!(i16 >= bool),
        (primitives::Type::Int32, PrimitiveOperation::GreaterThanOrEqual) => bin_op!(i32 >= bool),
        (primitives::Type::Int64, PrimitiveOperation::GreaterThanOrEqual) => bin_op!(i64 >= bool),
        (primitives::Type::Int128, PrimitiveOperation::GreaterThanOrEqual) => bin_op!(i128 >= bool),
        (primitives::Type::Float32, PrimitiveOperation::GreaterThanOrEqual) => bin_op!(f32 >= bool),
        (primitives::Type::Float64, PrimitiveOperation::GreaterThanOrEqual) => bin_op!(f64 >= bool),

        (primitives::Type::UInt8, PrimitiveOperation::LesserThan) => bin_op!(u8 < bool),
        (primitives::Type::UInt16, PrimitiveOperation::LesserThan) => bin_op!(u16 < bool),
        (primitives::Type::UInt32, PrimitiveOperation::LesserThan) => bin_op!(u32 < bool),
        (primitives::Type::UInt64, PrimitiveOperation::LesserThan) => bin_op!(u64 < bool),
        (primitives::Type::UInt128, PrimitiveOperation::LesserThan) => bin_op!(u128 < bool),
        (primitives::Type::Int8, PrimitiveOperation::LesserThan) => bin_op!(i8 < bool),
        (primitives::Type::Int16, PrimitiveOperation::LesserThan) => bin_op!(i16 < bool),
        (primitives::Type::Int32, PrimitiveOperation::LesserThan) => bin_op!(i32 < bool),
        (primitives::Type::Int64, PrimitiveOperation::LesserThan) => bin_op!(i64 < bool),
        (primitives::Type::Int128, PrimitiveOperation::LesserThan) => bin_op!(i128 < bool),
        (primitives::Type::Float32, PrimitiveOperation::LesserThan) => bin_op!(f32 < bool),
        (primitives::Type::Float64, PrimitiveOperation::LesserThan) => bin_op!(f64 < bool),

        (primitives::Type::UInt8, PrimitiveOperation::LesserThanOrEqual) => bin_op!(u8 <= bool),
        (primitives::Type::UInt16, PrimitiveOperation::LesserThanOrEqual) => bin_op!(u16 <= bool),
        (primitives::Type::UInt32, PrimitiveOperation::LesserThanOrEqual) => bin_op!(u32 <= bool),
        (primitives::Type::UInt64, PrimitiveOperation::LesserThanOrEqual) => bin_op!(u64 <= bool),
        (primitives::Type::UInt128, PrimitiveOperation::LesserThanOrEqual) => bin_op!(u128 <= bool),
        (primitives::Type::Int8, PrimitiveOperation::LesserThanOrEqual) => bin_op!(i8 <= bool),
        (primitives::Type::Int16, PrimitiveOperation::LesserThanOrEqual) => bin_op!(i16 <= bool),
        (primitives::Type::Int32, PrimitiveOperation::LesserThanOrEqual) => bin_op!(i32 <= bool),
        (primitives::Type::Int64, PrimitiveOperation::LesserThanOrEqual) => bin_op!(i64 <= bool),
        (primitives::Type::Int128, PrimitiveOperation::LesserThanOrEqual) => bin_op!(i128 <= bool),
        (primitives::Type::Float32, PrimitiveOperation::LesserThanOrEqual) => bin_op!(f32 <= bool),
        (primitives::Type::Float64, PrimitiveOperation::LesserThanOrEqual) => bin_op!(f64 <= bool),

        // -------------------------------------- Math --------------------------------------
        (primitives::Type::UInt8, PrimitiveOperation::Add) => bin_op!(u8 + u8),
        (primitives::Type::UInt16, PrimitiveOperation::Add) => bin_op!(u16 + u16),
        (primitives::Type::UInt32, PrimitiveOperation::Add) => bin_op!(u32 + u32),
        (primitives::Type::UInt64, PrimitiveOperation::Add) => bin_op!(u64 + u64),
        (primitives::Type::UInt128, PrimitiveOperation::Add) => bin_op!(u128 + u128),
        (primitives::Type::Int8, PrimitiveOperation::Add) => bin_op!(i8 + i8),
        (primitives::Type::Int16, PrimitiveOperation::Add) => bin_op!(i16 + i16),
        (primitives::Type::Int32, PrimitiveOperation::Add) => bin_op!(i32 + i32),
        (primitives::Type::Int64, PrimitiveOperation::Add) => bin_op!(i64 + i64),
        (primitives::Type::Int128, PrimitiveOperation::Add) => bin_op!(i128 + i128),
        (primitives::Type::Float32, PrimitiveOperation::Add) => bin_op!(f32 + f32),
        (primitives::Type::Float64, PrimitiveOperation::Add) => bin_op!(f64 + f64),

        (primitives::Type::UInt8, PrimitiveOperation::Subtract) => bin_op!(u8 - u8),
        (primitives::Type::UInt16, PrimitiveOperation::Subtract) => bin_op!(u16 - u16),
        (primitives::Type::UInt32, PrimitiveOperation::Subtract) => bin_op!(u32 - u32),
        (primitives::Type::UInt64, PrimitiveOperation::Subtract) => bin_op!(u64 - u64),
        (primitives::Type::UInt128, PrimitiveOperation::Subtract) => bin_op!(u128 - u128),
        (primitives::Type::Int8, PrimitiveOperation::Subtract) => bin_op!(i8 - i8),
        (primitives::Type::Int16, PrimitiveOperation::Subtract) => bin_op!(i16 - i16),
        (primitives::Type::Int32, PrimitiveOperation::Subtract) => bin_op!(i32 - i32),
        (primitives::Type::Int64, PrimitiveOperation::Subtract) => bin_op!(i64 - i64),
        (primitives::Type::Int128, PrimitiveOperation::Subtract) => bin_op!(i128 - i128),
        (primitives::Type::Float32, PrimitiveOperation::Subtract) => bin_op!(f32 - f32),
        (primitives::Type::Float64, PrimitiveOperation::Subtract) => bin_op!(f64 - f64),

        (primitives::Type::UInt8, PrimitiveOperation::Divide) => bin_op!(u8 / u8),
        (primitives::Type::UInt16, PrimitiveOperation::Divide) => bin_op!(u16 / u16),
        (primitives::Type::UInt32, PrimitiveOperation::Divide) => bin_op!(u32 / u32),
        (primitives::Type::UInt64, PrimitiveOperation::Divide) => bin_op!(u64 / u64),
        (primitives::Type::UInt128, PrimitiveOperation::Divide) => bin_op!(u128 / u128),
        (primitives::Type::Int8, PrimitiveOperation::Divide) => bin_op!(i8 / i8),
        (primitives::Type::Int16, PrimitiveOperation::Divide) => bin_op!(i16 / i16),
        (primitives::Type::Int32, PrimitiveOperation::Divide) => bin_op!(i32 / i32),
        (primitives::Type::Int64, PrimitiveOperation::Divide) => bin_op!(i64 / i64),
        (primitives::Type::Int128, PrimitiveOperation::Divide) => bin_op!(i128 / i128),
        (primitives::Type::Float32, PrimitiveOperation::Divide) => bin_op!(f32 / f32),
        (primitives::Type::Float64, PrimitiveOperation::Divide) => bin_op!(f64 / f64),

        (primitives::Type::UInt8, PrimitiveOperation::Multiply) => bin_op!(u8 * u8),
        (primitives::Type::UInt16, PrimitiveOperation::Multiply) => bin_op!(u16 * u16),
        (primitives::Type::UInt32, PrimitiveOperation::Multiply) => bin_op!(u32 * u32),
        (primitives::Type::UInt64, PrimitiveOperation::Multiply) => bin_op!(u64 * u64),
        (primitives::Type::UInt128, PrimitiveOperation::Multiply) => bin_op!(u128 * u128),
        (primitives::Type::Int8, PrimitiveOperation::Multiply) => bin_op!(i8 * i8),
        (primitives::Type::Int16, PrimitiveOperation::Multiply) => bin_op!(i16 * i16),
        (primitives::Type::Int32, PrimitiveOperation::Multiply) => bin_op!(i32 * i32),
        (primitives::Type::Int64, PrimitiveOperation::Multiply) => bin_op!(i64 * i64),
        (primitives::Type::Int128, PrimitiveOperation::Multiply) => bin_op!(i128 * i128),
        (primitives::Type::Float32, PrimitiveOperation::Multiply) => bin_op!(f32 * f32),
        (primitives::Type::Float64, PrimitiveOperation::Multiply) => bin_op!(f64 * f64),

        (primitives::Type::UInt8, PrimitiveOperation::Modulo) => bin_op!(u8 % u8),
        (primitives::Type::UInt16, PrimitiveOperation::Modulo) => bin_op!(u16 % u16),
        (primitives::Type::UInt32, PrimitiveOperation::Modulo) => bin_op!(u32 % u32),
        (primitives::Type::UInt64, PrimitiveOperation::Modulo) => bin_op!(u64 % u64),
        (primitives::Type::UInt128, PrimitiveOperation::Modulo) => bin_op!(u128 % u128),
        (primitives::Type::Int8, PrimitiveOperation::Modulo) => bin_op!(i8 % i8),
        (primitives::Type::Int16, PrimitiveOperation::Modulo) => bin_op!(i16 % i16),
        (primitives::Type::Int32, PrimitiveOperation::Modulo) => bin_op!(i32 % i32),
        (primitives::Type::Int64, PrimitiveOperation::Modulo) => bin_op!(i64 % i64),
        (primitives::Type::Int128, PrimitiveOperation::Modulo) => bin_op!(i128 % i128),
        (primitives::Type::Float32, PrimitiveOperation::Modulo) => bin_op!(f32 % f32),
        (primitives::Type::Float64, PrimitiveOperation::Modulo) => bin_op!(f64 % f64),

        (primitives::Type::Float32, PrimitiveOperation::Log) => fun_op!(f32 log f32),
        (primitives::Type::Float64, PrimitiveOperation::Log) => fun_op!(f64 log f64),
        (primitives::Type::Float32, PrimitiveOperation::Exp) => fun_op!(f32 powf f32),
        (primitives::Type::Float64, PrimitiveOperation::Exp) => fun_op!(f64 powf f64),

        // TODO For unsigneds, this shouldn't exist. But it does exist in $Number for now.
        (primitives::Type::UInt8, PrimitiveOperation::Negative) => un_op!(i8 -),
        (primitives::Type::UInt16, PrimitiveOperation::Negative) => un_op!(i16 -),
        (primitives::Type::UInt32, PrimitiveOperation::Negative) => un_op!(i32 -),
        (primitives::Type::UInt64, PrimitiveOperation::Negative) => un_op!(i64 -),
        (primitives::Type::UInt128, PrimitiveOperation::Negative) => un_op!(i128 -),
        (primitives::Type::Int8, PrimitiveOperation::Negative) => un_op!(i8 -),
        (primitives::Type::Int16, PrimitiveOperation::Negative) => un_op!(i16 -),
        (primitives::Type::Int32, PrimitiveOperation::Negative) => un_op!(i32 -),
        (primitives::Type::Int64, PrimitiveOperation::Negative) => un_op!(i64 -),
        (primitives::Type::Int128, PrimitiveOperation::Negative) => un_op!(i128 -),
        (primitives::Type::Float32, PrimitiveOperation::Negative) => un_op!(f32 -),
        (primitives::Type::Float64, PrimitiveOperation::Negative) => un_op!(f64 -),

        // -------------------------------------- Parse --------------------------------------
        (primitives::Type::UInt8, PrimitiveOperation::ParseIntString) => parse_op!(u8),
        (primitives::Type::UInt16, PrimitiveOperation::ParseIntString) => parse_op!(u16),
        (primitives::Type::UInt32, PrimitiveOperation::ParseIntString) => parse_op!(u32),
        (primitives::Type::UInt64, PrimitiveOperation::ParseIntString) => parse_op!(u64),
        (primitives::Type::UInt128, PrimitiveOperation::ParseIntString) => parse_op!(u128),
        (primitives::Type::Int8, PrimitiveOperation::ParseIntString) => parse_op!(i8),
        (primitives::Type::Int16, PrimitiveOperation::ParseIntString) => parse_op!(i16),
        (primitives::Type::Int32, PrimitiveOperation::ParseIntString) => parse_op!(i32),
        (primitives::Type::Int64, PrimitiveOperation::ParseIntString) => parse_op!(i64),
        (primitives::Type::Int128, PrimitiveOperation::ParseIntString) => parse_op!(i128),
        (primitives::Type::Float32, PrimitiveOperation::ParseIntString) => parse_op!(f32),
        (primitives::Type::Float64, PrimitiveOperation::ParseIntString) => parse_op!(f64),

        (primitives::Type::Float32, PrimitiveOperation::ParseRealString) => parse_op!(f32),
        (primitives::Type::Float64, PrimitiveOperation::ParseRealString) => parse_op!(f64),

        // -------------------------------------- ToString --------------------------------------
        (primitives::Type::Bool, PrimitiveOperation::ToString) => to_string_op!(bool),
        (primitives::Type::UInt8, PrimitiveOperation::ToString) => to_string_op!(u8),
        (primitives::Type::UInt16, PrimitiveOperation::ToString) => to_string_op!(u16),
        (primitives::Type::UInt32, PrimitiveOperation::ToString) => to_string_op!(u32),
        (primitives::Type::UInt64, PrimitiveOperation::ToString) => to_string_op!(u64),
        (primitives::Type::UInt128, PrimitiveOperation::ToString) => to_string_op!(u128),
        (primitives::Type::Int8, PrimitiveOperation::ToString) => to_string_op!(i8),
        (primitives::Type::Int16, PrimitiveOperation::ToString) => to_string_op!(i16),
        (primitives::Type::Int32, PrimitiveOperation::ToString) => to_string_op!(i32),
        (primitives::Type::Int64, PrimitiveOperation::ToString) => to_string_op!(i64),
        (primitives::Type::Int128, PrimitiveOperation::ToString) => to_string_op!(i128),
        (primitives::Type::Float32, PrimitiveOperation::ToString) => to_string_op!(f32),
        (primitives::Type::Float64, PrimitiveOperation::ToString) => to_string_op!(f64),

        _ => panic!("Unsupported primitive operation: {:?} on {:?}", operation, type_),
    }
}
