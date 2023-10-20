use std::alloc::{alloc, Layout};
use std::path::PathBuf;
use std::rc::Rc;
use monoteny_macro::{bin_op, fun_op, load_constant, parse_op, to_string_op, un_op};
use std::str::FromStr;
use guard::guard;
use uuid::Uuid;
use crate::error::RResult;
use crate::interpreter::{FunctionInterpreterImpl, Runtime};
use crate::interpreter::allocation::Value;
use crate::program::functions::FunctionHead;
use crate::program::global::{FunctionLogic, FunctionLogicDescriptor, PrimitiveOperation};
use crate::program::module::module_name;
use crate::program::primitives;
use crate::program::types::TypeUnit;

pub fn load(runtime: &mut Runtime) -> RResult<()> {
    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Monoteny files --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    runtime.repository.add("core", PathBuf::from("monoteny"));
    runtime.get_or_load_module(&module_name("core"))?;

    for function in runtime.source.module_by_name[&module_name("core.debug")].explicit_functions(&runtime.source) {
        let representation = &runtime.source.fn_representations[function];

        runtime.function_evaluators.insert(function.unwrap_id(), match representation.name.as_str() {
            "_write_line" => Rc::new(move |interpreter, expression_id, binding| {{
                unsafe {{
                    let args = interpreter.implementation.expression_tree.children[&expression_id].clone();
                    let arg = interpreter.evaluate(args[0]).unwrap();
                    println!("{}", *(arg.data as *const String));

                    None
                }}
            }}),
            "_exit_with_error" => Rc::new(move |interpreter, expression_id, binding| {{
                unsafe {{
                    let args = interpreter.implementation.expression_tree.children[&expression_id].clone();
                    let arg = interpreter.evaluate(args[0]).unwrap();

                    panic!("{}", *(arg.data as *const String));
                }}
            }}),
            _ => continue,
        });
    }

    for function in runtime.source.module_by_name[&module_name("core.transpilation")].explicit_functions(&runtime.source) {
        let representation = &runtime.source.fn_representations[function];

        runtime.function_evaluators.insert(
            function.function_id,
            Rc::new(move |interpreter, expression_id, binding| {
                unsafe {
                    let arguments = interpreter.evaluate_arguments(expression_id);

                    // This may cause a SIGSEV if the callback pointer is invalidated. This should not happen as long as
                    //  nobody owns a Transpiler object outside of its lifetime.
                    let transpiler_callback = *(arguments[0].data as *const &dyn Fn(Rc<FunctionHead>, &Runtime));

                    let arg = &arguments[1];
                    let arg_id = &interpreter.implementation.expression_tree.children[&expression_id][1];
                    let arg_type = interpreter.implementation.type_forest.get_unit(arg_id).unwrap();

                    match arg_type {
                        TypeUnit::Struct(s) => {
                            if !interpreter.runtime.source.function_traits.contains_key(s) {
                                panic!("Cannot transpile traits for now: {:?}", s);
                            }
                        }
                        _ => panic!(),
                    }

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

    for function in runtime.source.module_by_name[&module_name("core.bool")].explicit_functions(&runtime.source) {
        let representation = &runtime.source.fn_representations[function];

        runtime.function_evaluators.insert(function.unwrap_id(), match representation.name.as_str() {
            "true" => load_constant!(bool true),
            "false" => load_constant!(bool false),
            _ => continue,
        });
    }

    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Math --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    for function in runtime.source.module_by_name[&module_name("builtins")].explicit_functions(&runtime.source) {
        guard!(let Some(FunctionLogic::Descriptor(descriptor)) = runtime.source.fn_logic.get(function) else {
            continue;
        });

        runtime.function_evaluators.insert(function.unwrap_id(), match descriptor {
            FunctionLogicDescriptor::Stub => todo!(),
            FunctionLogicDescriptor::PrimitiveOperation { type_, operation } => {
                create_primitive_op(type_.clone(), operation.clone())
            }
            FunctionLogicDescriptor::Constructor(_) => todo!(),
            FunctionLogicDescriptor::GetMemberField(_, _) => todo!(),
            FunctionLogicDescriptor::SetMemberField(_, _) => todo!(),
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
        (primitives::Type::UInt(8), PrimitiveOperation::EqualTo) => bin_op!(u8 == bool),
        (primitives::Type::UInt(16), PrimitiveOperation::EqualTo) => bin_op!(u16 == bool),
        (primitives::Type::UInt(32), PrimitiveOperation::EqualTo) => bin_op!(u32 == bool),
        (primitives::Type::UInt(64), PrimitiveOperation::EqualTo) => bin_op!(u64 == bool),
        (primitives::Type::UInt(128), PrimitiveOperation::EqualTo) => bin_op!(u128 == bool),
        (primitives::Type::Int(8), PrimitiveOperation::EqualTo) => bin_op!(i8 == bool),
        (primitives::Type::Int(16), PrimitiveOperation::EqualTo) => bin_op!(i16 == bool),
        (primitives::Type::Int(32), PrimitiveOperation::EqualTo) => bin_op!(i32 == bool),
        (primitives::Type::Int(64), PrimitiveOperation::EqualTo) => bin_op!(i64 == bool),
        (primitives::Type::Int(128), PrimitiveOperation::EqualTo) => bin_op!(i128 == bool),
        (primitives::Type::Float(32), PrimitiveOperation::EqualTo) => bin_op!(f32 == bool),
        (primitives::Type::Float(64), PrimitiveOperation::EqualTo) => bin_op!(f64 == bool),

        (primitives::Type::Bool, PrimitiveOperation::NotEqualTo) => bin_op!(bool != bool),
        (primitives::Type::UInt(8), PrimitiveOperation::NotEqualTo) => bin_op!(u8 != bool),
        (primitives::Type::UInt(16), PrimitiveOperation::NotEqualTo) => bin_op!(u16 != bool),
        (primitives::Type::UInt(32), PrimitiveOperation::NotEqualTo) => bin_op!(u32 != bool),
        (primitives::Type::UInt(64), PrimitiveOperation::NotEqualTo) => bin_op!(u64 != bool),
        (primitives::Type::UInt(128), PrimitiveOperation::NotEqualTo) => bin_op!(u128 != bool),
        (primitives::Type::Int(8), PrimitiveOperation::NotEqualTo) => bin_op!(i8 != bool),
        (primitives::Type::Int(16), PrimitiveOperation::NotEqualTo) => bin_op!(i16 != bool),
        (primitives::Type::Int(32), PrimitiveOperation::NotEqualTo) => bin_op!(i32 != bool),
        (primitives::Type::Int(64), PrimitiveOperation::NotEqualTo) => bin_op!(i64 != bool),
        (primitives::Type::Int(128), PrimitiveOperation::NotEqualTo) => bin_op!(i128 != bool),
        (primitives::Type::Float(32), PrimitiveOperation::NotEqualTo) => bin_op!(f32 != bool),
        (primitives::Type::Float(64), PrimitiveOperation::NotEqualTo) => bin_op!(f64 != bool),

        (primitives::Type::UInt(8), PrimitiveOperation::GreaterThan) => bin_op!(u8 > bool),
        (primitives::Type::UInt(16), PrimitiveOperation::GreaterThan) => bin_op!(u16 > bool),
        (primitives::Type::UInt(32), PrimitiveOperation::GreaterThan) => bin_op!(u32 > bool),
        (primitives::Type::UInt(64), PrimitiveOperation::GreaterThan) => bin_op!(u64 > bool),
        (primitives::Type::UInt(128), PrimitiveOperation::GreaterThan) => bin_op!(u128 > bool),
        (primitives::Type::Int(8), PrimitiveOperation::GreaterThan) => bin_op!(i8 > bool),
        (primitives::Type::Int(16), PrimitiveOperation::GreaterThan) => bin_op!(i16 > bool),
        (primitives::Type::Int(32), PrimitiveOperation::GreaterThan) => bin_op!(i32 > bool),
        (primitives::Type::Int(64), PrimitiveOperation::GreaterThan) => bin_op!(i64 > bool),
        (primitives::Type::Int(128), PrimitiveOperation::GreaterThan) => bin_op!(i128 > bool),
        (primitives::Type::Float(32), PrimitiveOperation::GreaterThan) => bin_op!(f32 > bool),
        (primitives::Type::Float(64), PrimitiveOperation::GreaterThan) => bin_op!(f64 > bool),

        (primitives::Type::UInt(8), PrimitiveOperation::GreaterThanOrEqual) => bin_op!(u8 >= bool),
        (primitives::Type::UInt(16), PrimitiveOperation::GreaterThanOrEqual) => bin_op!(u16 >= bool),
        (primitives::Type::UInt(32), PrimitiveOperation::GreaterThanOrEqual) => bin_op!(u32 >= bool),
        (primitives::Type::UInt(64), PrimitiveOperation::GreaterThanOrEqual) => bin_op!(u64 >= bool),
        (primitives::Type::UInt(128), PrimitiveOperation::GreaterThanOrEqual) => bin_op!(u128 >= bool),
        (primitives::Type::Int(8), PrimitiveOperation::GreaterThanOrEqual) => bin_op!(i8 >= bool),
        (primitives::Type::Int(16), PrimitiveOperation::GreaterThanOrEqual) => bin_op!(i16 >= bool),
        (primitives::Type::Int(32), PrimitiveOperation::GreaterThanOrEqual) => bin_op!(i32 >= bool),
        (primitives::Type::Int(64), PrimitiveOperation::GreaterThanOrEqual) => bin_op!(i64 >= bool),
        (primitives::Type::Int(128), PrimitiveOperation::GreaterThanOrEqual) => bin_op!(i128 >= bool),
        (primitives::Type::Float(32), PrimitiveOperation::GreaterThanOrEqual) => bin_op!(f32 >= bool),
        (primitives::Type::Float(64), PrimitiveOperation::GreaterThanOrEqual) => bin_op!(f64 >= bool),

        (primitives::Type::UInt(8), PrimitiveOperation::LesserThan) => bin_op!(u8 < bool),
        (primitives::Type::UInt(16), PrimitiveOperation::LesserThan) => bin_op!(u16 < bool),
        (primitives::Type::UInt(32), PrimitiveOperation::LesserThan) => bin_op!(u32 < bool),
        (primitives::Type::UInt(64), PrimitiveOperation::LesserThan) => bin_op!(u64 < bool),
        (primitives::Type::UInt(128), PrimitiveOperation::LesserThan) => bin_op!(u128 < bool),
        (primitives::Type::Int(8), PrimitiveOperation::LesserThan) => bin_op!(i8 < bool),
        (primitives::Type::Int(16), PrimitiveOperation::LesserThan) => bin_op!(i16 < bool),
        (primitives::Type::Int(32), PrimitiveOperation::LesserThan) => bin_op!(i32 < bool),
        (primitives::Type::Int(64), PrimitiveOperation::LesserThan) => bin_op!(i64 < bool),
        (primitives::Type::Int(128), PrimitiveOperation::LesserThan) => bin_op!(i128 < bool),
        (primitives::Type::Float(32), PrimitiveOperation::LesserThan) => bin_op!(f32 < bool),
        (primitives::Type::Float(64), PrimitiveOperation::LesserThan) => bin_op!(f64 < bool),

        (primitives::Type::UInt(8), PrimitiveOperation::LesserThanOrEqual) => bin_op!(u8 <= bool),
        (primitives::Type::UInt(16), PrimitiveOperation::LesserThanOrEqual) => bin_op!(u16 <= bool),
        (primitives::Type::UInt(32), PrimitiveOperation::LesserThanOrEqual) => bin_op!(u32 <= bool),
        (primitives::Type::UInt(64), PrimitiveOperation::LesserThanOrEqual) => bin_op!(u64 <= bool),
        (primitives::Type::UInt(128), PrimitiveOperation::LesserThanOrEqual) => bin_op!(u128 <= bool),
        (primitives::Type::Int(8), PrimitiveOperation::LesserThanOrEqual) => bin_op!(i8 <= bool),
        (primitives::Type::Int(16), PrimitiveOperation::LesserThanOrEqual) => bin_op!(i16 <= bool),
        (primitives::Type::Int(32), PrimitiveOperation::LesserThanOrEqual) => bin_op!(i32 <= bool),
        (primitives::Type::Int(64), PrimitiveOperation::LesserThanOrEqual) => bin_op!(i64 <= bool),
        (primitives::Type::Int(128), PrimitiveOperation::LesserThanOrEqual) => bin_op!(i128 <= bool),
        (primitives::Type::Float(32), PrimitiveOperation::LesserThanOrEqual) => bin_op!(f32 <= bool),
        (primitives::Type::Float(64), PrimitiveOperation::LesserThanOrEqual) => bin_op!(f64 <= bool),

        // -------------------------------------- Math --------------------------------------
        (primitives::Type::UInt(8), PrimitiveOperation::Add) => bin_op!(u8 + u8),
        (primitives::Type::UInt(16), PrimitiveOperation::Add) => bin_op!(u16 + u16),
        (primitives::Type::UInt(32), PrimitiveOperation::Add) => bin_op!(u32 + u32),
        (primitives::Type::UInt(64), PrimitiveOperation::Add) => bin_op!(u64 + u64),
        (primitives::Type::UInt(128), PrimitiveOperation::Add) => bin_op!(u128 + u128),
        (primitives::Type::Int(8), PrimitiveOperation::Add) => bin_op!(i8 + i8),
        (primitives::Type::Int(16), PrimitiveOperation::Add) => bin_op!(i16 + i16),
        (primitives::Type::Int(32), PrimitiveOperation::Add) => bin_op!(i32 + i32),
        (primitives::Type::Int(64), PrimitiveOperation::Add) => bin_op!(i64 + i64),
        (primitives::Type::Int(128), PrimitiveOperation::Add) => bin_op!(i128 + i128),
        (primitives::Type::Float(32), PrimitiveOperation::Add) => bin_op!(f32 + f32),
        (primitives::Type::Float(64), PrimitiveOperation::Add) => bin_op!(f64 + f64),

        (primitives::Type::UInt(8), PrimitiveOperation::Subtract) => bin_op!(u8 - u8),
        (primitives::Type::UInt(16), PrimitiveOperation::Subtract) => bin_op!(u16 - u16),
        (primitives::Type::UInt(32), PrimitiveOperation::Subtract) => bin_op!(u32 - u32),
        (primitives::Type::UInt(64), PrimitiveOperation::Subtract) => bin_op!(u64 - u64),
        (primitives::Type::UInt(128), PrimitiveOperation::Subtract) => bin_op!(u128 - u128),
        (primitives::Type::Int(8), PrimitiveOperation::Subtract) => bin_op!(i8 - i8),
        (primitives::Type::Int(16), PrimitiveOperation::Subtract) => bin_op!(i16 - i16),
        (primitives::Type::Int(32), PrimitiveOperation::Subtract) => bin_op!(i32 - i32),
        (primitives::Type::Int(64), PrimitiveOperation::Subtract) => bin_op!(i64 - i64),
        (primitives::Type::Int(128), PrimitiveOperation::Subtract) => bin_op!(i128 - i128),
        (primitives::Type::Float(32), PrimitiveOperation::Subtract) => bin_op!(f32 - f32),
        (primitives::Type::Float(64), PrimitiveOperation::Subtract) => bin_op!(f64 - f64),

        (primitives::Type::UInt(8), PrimitiveOperation::Divide) => bin_op!(u8 / u8),
        (primitives::Type::UInt(16), PrimitiveOperation::Divide) => bin_op!(u16 / u16),
        (primitives::Type::UInt(32), PrimitiveOperation::Divide) => bin_op!(u32 / u32),
        (primitives::Type::UInt(64), PrimitiveOperation::Divide) => bin_op!(u64 / u64),
        (primitives::Type::UInt(128), PrimitiveOperation::Divide) => bin_op!(u128 / u128),
        (primitives::Type::Int(8), PrimitiveOperation::Divide) => bin_op!(i8 / i8),
        (primitives::Type::Int(16), PrimitiveOperation::Divide) => bin_op!(i16 / i16),
        (primitives::Type::Int(32), PrimitiveOperation::Divide) => bin_op!(i32 / i32),
        (primitives::Type::Int(64), PrimitiveOperation::Divide) => bin_op!(i64 / i64),
        (primitives::Type::Int(128), PrimitiveOperation::Divide) => bin_op!(i128 / i128),
        (primitives::Type::Float(32), PrimitiveOperation::Divide) => bin_op!(f32 / f32),
        (primitives::Type::Float(64), PrimitiveOperation::Divide) => bin_op!(f64 / f64),

        (primitives::Type::UInt(8), PrimitiveOperation::Multiply) => bin_op!(u8 * u8),
        (primitives::Type::UInt(16), PrimitiveOperation::Multiply) => bin_op!(u16 * u16),
        (primitives::Type::UInt(32), PrimitiveOperation::Multiply) => bin_op!(u32 * u32),
        (primitives::Type::UInt(64), PrimitiveOperation::Multiply) => bin_op!(u64 * u64),
        (primitives::Type::UInt(128), PrimitiveOperation::Multiply) => bin_op!(u128 * u128),
        (primitives::Type::Int(8), PrimitiveOperation::Multiply) => bin_op!(i8 * i8),
        (primitives::Type::Int(16), PrimitiveOperation::Multiply) => bin_op!(i16 * i16),
        (primitives::Type::Int(32), PrimitiveOperation::Multiply) => bin_op!(i32 * i32),
        (primitives::Type::Int(64), PrimitiveOperation::Multiply) => bin_op!(i64 * i64),
        (primitives::Type::Int(128), PrimitiveOperation::Multiply) => bin_op!(i128 * i128),
        (primitives::Type::Float(32), PrimitiveOperation::Multiply) => bin_op!(f32 * f32),
        (primitives::Type::Float(64), PrimitiveOperation::Multiply) => bin_op!(f64 * f64),

        (primitives::Type::UInt(8), PrimitiveOperation::Modulo) => bin_op!(u8 % u8),
        (primitives::Type::UInt(16), PrimitiveOperation::Modulo) => bin_op!(u16 % u16),
        (primitives::Type::UInt(32), PrimitiveOperation::Modulo) => bin_op!(u32 % u32),
        (primitives::Type::UInt(64), PrimitiveOperation::Modulo) => bin_op!(u64 % u64),
        (primitives::Type::UInt(128), PrimitiveOperation::Modulo) => bin_op!(u128 % u128),
        (primitives::Type::Int(8), PrimitiveOperation::Modulo) => bin_op!(i8 % i8),
        (primitives::Type::Int(16), PrimitiveOperation::Modulo) => bin_op!(i16 % i16),
        (primitives::Type::Int(32), PrimitiveOperation::Modulo) => bin_op!(i32 % i32),
        (primitives::Type::Int(64), PrimitiveOperation::Modulo) => bin_op!(i64 % i64),
        (primitives::Type::Int(128), PrimitiveOperation::Modulo) => bin_op!(i128 % i128),
        (primitives::Type::Float(32), PrimitiveOperation::Modulo) => bin_op!(f32 % f32),
        (primitives::Type::Float(64), PrimitiveOperation::Modulo) => bin_op!(f64 % f64),

        (primitives::Type::Float(32), PrimitiveOperation::Log) => fun_op!(f32 log f32),
        (primitives::Type::Float(64), PrimitiveOperation::Log) => fun_op!(f64 log f64),
        (primitives::Type::Float(32), PrimitiveOperation::Exp) => fun_op!(f32 powf f32),
        (primitives::Type::Float(64), PrimitiveOperation::Exp) => fun_op!(f64 powf f64),

        // TODO For unsigneds, this shouldn't exist. But it does exist in $Number for now.
        (primitives::Type::UInt(8), PrimitiveOperation::Negative) => un_op!(i8 -),
        (primitives::Type::UInt(16), PrimitiveOperation::Negative) => un_op!(i16 -),
        (primitives::Type::UInt(32), PrimitiveOperation::Negative) => un_op!(i32 -),
        (primitives::Type::UInt(64), PrimitiveOperation::Negative) => un_op!(i64 -),
        (primitives::Type::UInt(128), PrimitiveOperation::Negative) => un_op!(i128 -),
        (primitives::Type::Int(8), PrimitiveOperation::Negative) => un_op!(i8 -),
        (primitives::Type::Int(16), PrimitiveOperation::Negative) => un_op!(i16 -),
        (primitives::Type::Int(32), PrimitiveOperation::Negative) => un_op!(i32 -),
        (primitives::Type::Int(64), PrimitiveOperation::Negative) => un_op!(i64 -),
        (primitives::Type::Int(128), PrimitiveOperation::Negative) => un_op!(i128 -),
        (primitives::Type::Float(32), PrimitiveOperation::Negative) => un_op!(f32 -),
        (primitives::Type::Float(64), PrimitiveOperation::Negative) => un_op!(f64 -),

        // -------------------------------------- Parse --------------------------------------
        (primitives::Type::UInt(8), PrimitiveOperation::ParseIntString) => parse_op!(u8),
        (primitives::Type::UInt(16), PrimitiveOperation::ParseIntString) => parse_op!(u16),
        (primitives::Type::UInt(32), PrimitiveOperation::ParseIntString) => parse_op!(u32),
        (primitives::Type::UInt(64), PrimitiveOperation::ParseIntString) => parse_op!(u64),
        (primitives::Type::UInt(128), PrimitiveOperation::ParseIntString) => parse_op!(u128),
        (primitives::Type::Int(8), PrimitiveOperation::ParseIntString) => parse_op!(i8),
        (primitives::Type::Int(16), PrimitiveOperation::ParseIntString) => parse_op!(i16),
        (primitives::Type::Int(32), PrimitiveOperation::ParseIntString) => parse_op!(i32),
        (primitives::Type::Int(64), PrimitiveOperation::ParseIntString) => parse_op!(i64),
        (primitives::Type::Int(128), PrimitiveOperation::ParseIntString) => parse_op!(i128),
        (primitives::Type::Float(32), PrimitiveOperation::ParseIntString) => parse_op!(f32),
        (primitives::Type::Float(64), PrimitiveOperation::ParseIntString) => parse_op!(f64),

        (primitives::Type::Float(32), PrimitiveOperation::ParseRealString) => parse_op!(f32),
        (primitives::Type::Float(64), PrimitiveOperation::ParseRealString) => parse_op!(f64),

        // -------------------------------------- ToString --------------------------------------
        (primitives::Type::Bool, PrimitiveOperation::ToString) => to_string_op!(bool),
        (primitives::Type::UInt(8), PrimitiveOperation::ToString) => to_string_op!(u8),
        (primitives::Type::UInt(16), PrimitiveOperation::ToString) => to_string_op!(u16),
        (primitives::Type::UInt(32), PrimitiveOperation::ToString) => to_string_op!(u32),
        (primitives::Type::UInt(64), PrimitiveOperation::ToString) => to_string_op!(u64),
        (primitives::Type::UInt(128), PrimitiveOperation::ToString) => to_string_op!(u128),
        (primitives::Type::Int(8), PrimitiveOperation::ToString) => to_string_op!(i8),
        (primitives::Type::Int(16), PrimitiveOperation::ToString) => to_string_op!(i16),
        (primitives::Type::Int(32), PrimitiveOperation::ToString) => to_string_op!(i32),
        (primitives::Type::Int(64), PrimitiveOperation::ToString) => to_string_op!(i64),
        (primitives::Type::Int(128), PrimitiveOperation::ToString) => to_string_op!(i128),
        (primitives::Type::Float(32), PrimitiveOperation::ToString) => to_string_op!(f32),
        (primitives::Type::Float(64), PrimitiveOperation::ToString) => to_string_op!(f64),

        _ => panic!("Unsupported primitive operation: {:?} on {:?}", operation, type_),
    }
}
