use std::alloc::{alloc, Layout};
use std::collections::HashMap;
use std::rc::Rc;
use guard::guard;
use itertools::Itertools;
use monoteny_macro::{bin_op, parse_op, un_op, fun_op};
use std::str::FromStr;
use uuid::Uuid;
use crate::interpreter::{FunctionInterpreter, FunctionInterpreterImpl, Value};
use crate::program::builtins::Builtins;
use crate::program::functions::FunctionPointer;
use crate::program::primitives;
use crate::program::types::TypeUnit;

pub fn make_evaluators(builtins: &Builtins) -> HashMap<Uuid, FunctionInterpreterImpl> {
    let mut map: HashMap<Uuid, FunctionInterpreterImpl> = HashMap::new();

    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Math --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    // -------------------------------------- Add --------------------------------------
    map.insert(builtins.primitives.add[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 + u8));
    map.insert(builtins.primitives.add[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 + u16));
    map.insert(builtins.primitives.add[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 + u32));
    map.insert(builtins.primitives.add[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 + u64));
    map.insert(builtins.primitives.add[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 + u128));
    map.insert(builtins.primitives.add[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 + i8));
    map.insert(builtins.primitives.add[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 + i16));
    map.insert(builtins.primitives.add[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 + i32));
    map.insert(builtins.primitives.add[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 + i64));
    map.insert(builtins.primitives.add[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 + i128));
    map.insert(builtins.primitives.add[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 + f32));
    map.insert(builtins.primitives.add[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 + f64));

    // -------------------------------------- Subtract --------------------------------------
    map.insert(builtins.primitives.subtract[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 - u8));
    map.insert(builtins.primitives.subtract[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 - u16));
    map.insert(builtins.primitives.subtract[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 - u32));
    map.insert(builtins.primitives.subtract[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 - u64));
    map.insert(builtins.primitives.subtract[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 - u128));
    map.insert(builtins.primitives.subtract[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 - i8));
    map.insert(builtins.primitives.subtract[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 - i16));
    map.insert(builtins.primitives.subtract[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 - i32));
    map.insert(builtins.primitives.subtract[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 - i64));
    map.insert(builtins.primitives.subtract[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 - i128));
    map.insert(builtins.primitives.subtract[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 - f32));
    map.insert(builtins.primitives.subtract[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 - f64));

    // -------------------------------------- Divide --------------------------------------
    map.insert(builtins.primitives.divide[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 / u8));
    map.insert(builtins.primitives.divide[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 / u16));
    map.insert(builtins.primitives.divide[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 / u32));
    map.insert(builtins.primitives.divide[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 / u64));
    map.insert(builtins.primitives.divide[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 / u128));
    map.insert(builtins.primitives.divide[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 / i8));
    map.insert(builtins.primitives.divide[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 / i16));
    map.insert(builtins.primitives.divide[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 / i32));
    map.insert(builtins.primitives.divide[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 / i64));
    map.insert(builtins.primitives.divide[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 / i128));
    map.insert(builtins.primitives.divide[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 / f32));
    map.insert(builtins.primitives.divide[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 / f64));

    // -------------------------------------- Multiply --------------------------------------
    map.insert(builtins.primitives.multiply[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 * u8));
    map.insert(builtins.primitives.multiply[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 * u16));
    map.insert(builtins.primitives.multiply[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 * u32));
    map.insert(builtins.primitives.multiply[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 * u64));
    map.insert(builtins.primitives.multiply[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 * u128));
    map.insert(builtins.primitives.multiply[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 * i8));
    map.insert(builtins.primitives.multiply[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 * i16));
    map.insert(builtins.primitives.multiply[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 * i32));
    map.insert(builtins.primitives.multiply[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 * i64));
    map.insert(builtins.primitives.multiply[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 * i128));
    map.insert(builtins.primitives.multiply[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 * f32));
    map.insert(builtins.primitives.multiply[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 * f64));

    // -------------------------------------- Multiply --------------------------------------
    map.insert(builtins.primitives.modulo[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 % u8));
    map.insert(builtins.primitives.modulo[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 % u16));
    map.insert(builtins.primitives.modulo[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 % u32));
    map.insert(builtins.primitives.modulo[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 % u64));
    map.insert(builtins.primitives.modulo[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 % u128));
    map.insert(builtins.primitives.modulo[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 % i8));
    map.insert(builtins.primitives.modulo[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 % i16));
    map.insert(builtins.primitives.modulo[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 % i32));
    map.insert(builtins.primitives.modulo[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 % i64));
    map.insert(builtins.primitives.modulo[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 % i128));
    map.insert(builtins.primitives.modulo[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 % f32));
    map.insert(builtins.primitives.modulo[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 % f64));

    // -------------------------------------- Floats --------------------------------------
    map.insert(builtins.primitives.exponent[&primitives::Type::Float32].unwrap_id(), fun_op!(f32 powf f32));
    map.insert(builtins.primitives.exponent[&primitives::Type::Float64].unwrap_id(), fun_op!(f64 powf f64));
    map.insert(builtins.primitives.logarithm[&primitives::Type::Float32].unwrap_id(), fun_op!(f32 log f32));
    map.insert(builtins.primitives.logarithm[&primitives::Type::Float64].unwrap_id(), fun_op!(f64 log f64));

    // -------------------------------------- Positive --------------------------------------
    // TODO Either remove positive op, or solve it as no-op. Either way, rust has no 'positive' op.

    // -------------------------------------- Negative --------------------------------------
    map.insert(builtins.primitives.negative[&primitives::Type::Int8].unwrap_id(), un_op!(i8 -));
    map.insert(builtins.primitives.negative[&primitives::Type::Int16].unwrap_id(), un_op!(i16 -));
    map.insert(builtins.primitives.negative[&primitives::Type::Int32].unwrap_id(), un_op!(i32 -));
    map.insert(builtins.primitives.negative[&primitives::Type::Int64].unwrap_id(), un_op!(i64 -));
    map.insert(builtins.primitives.negative[&primitives::Type::Int128].unwrap_id(), un_op!(i128 -));
    map.insert(builtins.primitives.negative[&primitives::Type::Float32].unwrap_id(), un_op!(f32 -));
    map.insert(builtins.primitives.negative[&primitives::Type::Float64].unwrap_id(), un_op!(f64 -));

    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Parsing --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    // -------------------------------------- Parse Int Literal --------------------------------------
    map.insert(builtins.primitives.parse_int_literal[&primitives::Type::UInt8].unwrap_id(), parse_op!(u8));
    map.insert(builtins.primitives.parse_int_literal[&primitives::Type::UInt16].unwrap_id(), parse_op!(u16));
    map.insert(builtins.primitives.parse_int_literal[&primitives::Type::UInt32].unwrap_id(), parse_op!(u32));
    map.insert(builtins.primitives.parse_int_literal[&primitives::Type::UInt64].unwrap_id(), parse_op!(u64));
    map.insert(builtins.primitives.parse_int_literal[&primitives::Type::UInt128].unwrap_id(), parse_op!(u128));
    map.insert(builtins.primitives.parse_int_literal[&primitives::Type::Int8].unwrap_id(), parse_op!(i8));
    map.insert(builtins.primitives.parse_int_literal[&primitives::Type::Int16].unwrap_id(), parse_op!(i16));
    map.insert(builtins.primitives.parse_int_literal[&primitives::Type::Int32].unwrap_id(), parse_op!(i32));
    map.insert(builtins.primitives.parse_int_literal[&primitives::Type::Int64].unwrap_id(), parse_op!(i64));
    map.insert(builtins.primitives.parse_int_literal[&primitives::Type::Int128].unwrap_id(), parse_op!(i128));
    map.insert(builtins.primitives.parse_int_literal[&primitives::Type::Float32].unwrap_id(), parse_op!(f32));
    map.insert(builtins.primitives.parse_int_literal[&primitives::Type::Float64].unwrap_id(), parse_op!(f64));

    // -------------------------------------- Parse Float Literal --------------------------------------
    map.insert(builtins.primitives.parse_float_literal[&primitives::Type::Float32].unwrap_id(), parse_op!(f32));
    map.insert(builtins.primitives.parse_float_literal[&primitives::Type::Float64].unwrap_id(), parse_op!(f64));


    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Boolean --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    // -------------------------------------- Bool -> Bool --------------------------------------
    map.insert(builtins.primitives.and.unwrap_id(), bin_op!(bool && bool));
    map.insert(builtins.primitives.or.unwrap_id(), bin_op!(bool || bool));
    map.insert(builtins.primitives.not.unwrap_id(), un_op!(bool !));

    // -------------------------------------- Equals --------------------------------------
    map.insert(builtins.primitives.equal_to[&primitives::Type::Bool].unwrap_id(), bin_op!(bool == bool));
    map.insert(builtins.primitives.equal_to[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 == bool));
    map.insert(builtins.primitives.equal_to[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 == bool));
    map.insert(builtins.primitives.equal_to[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 == bool));
    map.insert(builtins.primitives.equal_to[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 == bool));
    map.insert(builtins.primitives.equal_to[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 == bool));
    map.insert(builtins.primitives.equal_to[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 == bool));
    map.insert(builtins.primitives.equal_to[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 == bool));
    map.insert(builtins.primitives.equal_to[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 == bool));
    map.insert(builtins.primitives.equal_to[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 == bool));
    map.insert(builtins.primitives.equal_to[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 == bool));
    map.insert(builtins.primitives.equal_to[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 == bool));
    map.insert(builtins.primitives.equal_to[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 == bool));

    // -------------------------------------- Not Equals --------------------------------------
    map.insert(builtins.primitives.not_equal_to[&primitives::Type::Bool].unwrap_id(), bin_op!(bool != bool));
    map.insert(builtins.primitives.not_equal_to[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 != bool));
    map.insert(builtins.primitives.not_equal_to[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 != bool));
    map.insert(builtins.primitives.not_equal_to[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 != bool));
    map.insert(builtins.primitives.not_equal_to[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 != bool));
    map.insert(builtins.primitives.not_equal_to[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 != bool));
    map.insert(builtins.primitives.not_equal_to[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 != bool));
    map.insert(builtins.primitives.not_equal_to[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 != bool));
    map.insert(builtins.primitives.not_equal_to[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 != bool));
    map.insert(builtins.primitives.not_equal_to[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 != bool));
    map.insert(builtins.primitives.not_equal_to[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 != bool));
    map.insert(builtins.primitives.not_equal_to[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 != bool));
    map.insert(builtins.primitives.not_equal_to[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 != bool));

    // -------------------------------------- Greater Than --------------------------------------
    map.insert(builtins.primitives.greater_than[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 > bool));
    map.insert(builtins.primitives.greater_than[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 > bool));
    map.insert(builtins.primitives.greater_than[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 > bool));
    map.insert(builtins.primitives.greater_than[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 > bool));
    map.insert(builtins.primitives.greater_than[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 > bool));
    map.insert(builtins.primitives.greater_than[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 > bool));
    map.insert(builtins.primitives.greater_than[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 > bool));
    map.insert(builtins.primitives.greater_than[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 > bool));
    map.insert(builtins.primitives.greater_than[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 > bool));
    map.insert(builtins.primitives.greater_than[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 > bool));
    map.insert(builtins.primitives.greater_than[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 > bool));
    map.insert(builtins.primitives.greater_than[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 > bool));

    // -------------------------------------- Greater Than Or Equal To --------------------------------------
    map.insert(builtins.primitives.greater_than_or_equal_to[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 >= bool));
    map.insert(builtins.primitives.greater_than_or_equal_to[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 >= bool));
    map.insert(builtins.primitives.greater_than_or_equal_to[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 >= bool));
    map.insert(builtins.primitives.greater_than_or_equal_to[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 >= bool));
    map.insert(builtins.primitives.greater_than_or_equal_to[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 >= bool));
    map.insert(builtins.primitives.greater_than_or_equal_to[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 >= bool));
    map.insert(builtins.primitives.greater_than_or_equal_to[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 >= bool));
    map.insert(builtins.primitives.greater_than_or_equal_to[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 >= bool));
    map.insert(builtins.primitives.greater_than_or_equal_to[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 >= bool));
    map.insert(builtins.primitives.greater_than_or_equal_to[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 >= bool));
    map.insert(builtins.primitives.greater_than_or_equal_to[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 >= bool));
    map.insert(builtins.primitives.greater_than_or_equal_to[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 >= bool));

    // -------------------------------------- Lesser Than --------------------------------------
    map.insert(builtins.primitives.lesser_than[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 < bool));
    map.insert(builtins.primitives.lesser_than[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 < bool));
    map.insert(builtins.primitives.lesser_than[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 < bool));
    map.insert(builtins.primitives.lesser_than[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 < bool));
    map.insert(builtins.primitives.lesser_than[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 < bool));
    map.insert(builtins.primitives.lesser_than[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 < bool));
    map.insert(builtins.primitives.lesser_than[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 < bool));
    map.insert(builtins.primitives.lesser_than[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 < bool));
    map.insert(builtins.primitives.lesser_than[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 < bool));
    map.insert(builtins.primitives.lesser_than[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 < bool));
    map.insert(builtins.primitives.lesser_than[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 < bool));
    map.insert(builtins.primitives.lesser_than[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 < bool));

    // -------------------------------------- Lesser Than --------------------------------------
    map.insert(builtins.primitives.lesser_than_or_equal_to[&primitives::Type::UInt8].unwrap_id(), bin_op!(u8 <= bool));
    map.insert(builtins.primitives.lesser_than_or_equal_to[&primitives::Type::UInt16].unwrap_id(), bin_op!(u16 <= bool));
    map.insert(builtins.primitives.lesser_than_or_equal_to[&primitives::Type::UInt32].unwrap_id(), bin_op!(u32 <= bool));
    map.insert(builtins.primitives.lesser_than_or_equal_to[&primitives::Type::UInt64].unwrap_id(), bin_op!(u64 <= bool));
    map.insert(builtins.primitives.lesser_than_or_equal_to[&primitives::Type::UInt128].unwrap_id(), bin_op!(u128 <= bool));
    map.insert(builtins.primitives.lesser_than_or_equal_to[&primitives::Type::Int8].unwrap_id(), bin_op!(i8 <= bool));
    map.insert(builtins.primitives.lesser_than_or_equal_to[&primitives::Type::Int16].unwrap_id(), bin_op!(i16 <= bool));
    map.insert(builtins.primitives.lesser_than_or_equal_to[&primitives::Type::Int32].unwrap_id(), bin_op!(i32 <= bool));
    map.insert(builtins.primitives.lesser_than_or_equal_to[&primitives::Type::Int64].unwrap_id(), bin_op!(i64 <= bool));
    map.insert(builtins.primitives.lesser_than_or_equal_to[&primitives::Type::Int128].unwrap_id(), bin_op!(i128 <= bool));
    map.insert(builtins.primitives.lesser_than_or_equal_to[&primitives::Type::Float32].unwrap_id(), bin_op!(f32 <= bool));
    map.insert(builtins.primitives.lesser_than_or_equal_to[&primitives::Type::Float64].unwrap_id(), bin_op!(f64 <= bool));

    // -------------------------------------- ------ --------------------------------------
    // -------------------------------------- Common --------------------------------------
    // -------------------------------------- ------ --------------------------------------

    map.insert(builtins.debug.print.unwrap_id(), Box::new(|interpreter, expression_id, binding| {
        unsafe {
            let arg_id = &interpreter.function.expression_forest.arguments[expression_id][0];
            let arg = interpreter.evaluate(arg_id).unwrap();
            let arg_type = interpreter.function.type_forest.get_unit(arg_id).unwrap();

            // TODO Instead, introduce a ToString trait that can be called, with each getting their own function to fit it.
            //  If not implemented, dump the type instead.
            println!("{}", match arg_type {
                TypeUnit::Struct(s) => {
                    if s == &interpreter.builtins.traits.String {
                        (*(arg.data as *const String)).clone()
                    }
                    else {
                        panic!()
                    }
                },
                TypeUnit::Primitive(primitives::Type::Bool) => (*(arg.data as *const bool)).to_string(),
                TypeUnit::Primitive(primitives::Type::Int8) => (*(arg.data as *const i8)).to_string(),
                TypeUnit::Primitive(primitives::Type::Int16) => (*(arg.data as *const i16)).to_string(),
                TypeUnit::Primitive(primitives::Type::Int32) => (*(arg.data as *const i32)).to_string(),
                TypeUnit::Primitive(primitives::Type::Int64) => (*(arg.data as *const i64)).to_string(),
                TypeUnit::Primitive(primitives::Type::Int128) => (*(arg.data as *const i128)).to_string(),
                TypeUnit::Primitive(primitives::Type::UInt8) => (*(arg.data as *const u8)).to_string(),
                TypeUnit::Primitive(primitives::Type::UInt16) => (*(arg.data as *const u16)).to_string(),
                TypeUnit::Primitive(primitives::Type::UInt32) => (*(arg.data as *const u32)).to_string(),
                TypeUnit::Primitive(primitives::Type::UInt64) => (*(arg.data as *const u64)).to_string(),
                TypeUnit::Primitive(primitives::Type::UInt128) => (*(arg.data as *const u128)).to_string(),
                TypeUnit::Primitive(primitives::Type::Float32) => (*(arg.data as *const f32)).to_string(),
                TypeUnit::Primitive(primitives::Type::Float64) => (*(arg.data as *const f64)).to_string(),
                _ => panic!(),
            });

            return None;
        }
    }));

    let bool_layout = Layout::new::<bool>();
    map.insert(builtins.common.true_.unwrap_id(), Box::new(move |interpreter, expression_id, binding| {
        unsafe {
            let ptr = alloc(bool_layout);
            *(ptr as *mut bool) = true;
            return Some(Value { data: ptr, layout: bool_layout })
        }
    }));
    map.insert(builtins.common.false_.unwrap_id(), Box::new(move |interpreter, expression_id, binding| {
        unsafe {
            let ptr = alloc(bool_layout);
            *(ptr as *mut bool) = false;
            return Some(Value { data: ptr, layout: bool_layout })
        }
    }));

    map
}
